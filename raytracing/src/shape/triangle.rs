use glam::Vec3;

use crate::{
    material::MaterialId,
    math::{bounds::Bounds, point::Point},
    ray::Ray,
};

use super::{
    local_info, shape::RayIntersection, FullIntersectionResult, IntersectionResult,
    MinIntersectionResult, Shape,
};

/// Describes the winding of a triangle.
#[derive(Debug, Default)]
pub enum Winding {
    #[default]
    ClockWise,
    CounterClockWise,
}

/// Gather information to create a triangle.
///
/// Use the [TriangleBuilder::build] method to compute additional data and build a [Triangle]
#[derive(Debug)]
pub struct TriangleBuilder {
    pub vertices: [Point; 3],
    pub winding: Winding,
}

/// A simple triangle Shape
pub struct Triangle {
    pub vertices: [Point; 3],
    pub normals: [Vec3; 3],
    pub material: MaterialId,
}

impl TriangleBuilder {
    pub fn build(self, material: MaterialId) -> Triangle {
        let vertices = self.vertices;
        let winding = self.winding;

        let a = vertices[1] - vertices[0];
        let b = vertices[2] - vertices[0];
        let winding_sign_correction = match winding {
            Winding::ClockWise => 1.0,
            Winding::CounterClockWise => -1.0,
        };
        let normal = winding_sign_correction * a.cross(b).normalize_or_zero();
        Triangle {
            vertices,
            normals: [normal, normal, normal],
            material,
        }
    }
}

/// A private type that stores the result of the Möller-Trumbore algorithm
enum MollerTrumboreResult {
    Result { u: f32, v: f32, t: f32 },
    NoResult,
}

impl MollerTrumboreResult {
    fn moller_trumbore(vertices: [Point; 3], ray: Ray) -> Self {
        #[allow(non_snake_case)]
        let M = glam::mat3(
            vertices[2] - vertices[0],
            vertices[2] - vertices[1],
            ray.direction,
        );

        if M.determinant() == 0.0 {
            MollerTrumboreResult::NoResult
        } else {
            let [u, v, t] = M.inverse().mul_vec3(vertices[2] - ray.origin).to_array();
            MollerTrumboreResult::Result { u, v, t }
        }
    }

    fn from_triangle(triangle: &Triangle, ray: Ray) -> Self {
        Self::moller_trumbore(triangle.vertices, ray)
    }
}

impl Shape for Triangle {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        match MollerTrumboreResult::from_triangle(self, ray) {
            MollerTrumboreResult::Result { u, v, t } => {
                let w = 1.0 - u - v;
                if (0.0..=1.0).contains(&u)
                    && (0.0..=1.0).contains(&v)
                    && (0.0..=1.0).contains(&w)
                    && ray.range().contains(&t)
                {
                    let pos = ray.at(t);
                    let normal = u * self.normals[0] + v * self.normals[1] + w * self.normals[2];
                    IntersectionResult::Intersection(RayIntersection {
                        t,
                        local_info: local_info::Full {
                            pos,
                            normal,
                            material: self.material,
                            uv: [u, v],
                        },
                    })
                } else {
                    IntersectionResult::NoIntersection
                }
            }
            _ => IntersectionResult::NoIntersection,
        }
    }

    fn intersect_bare(&self, ray: Ray) -> MinIntersectionResult {
        match MollerTrumboreResult::from_triangle(self, ray) {
            MollerTrumboreResult::Result { u, v, t } => {
                if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) && ray.range().contains(&t)
                {
                    let pos = ray.at(t);
                    IntersectionResult::Intersection(RayIntersection {
                        t,
                        local_info: local_info::Minimum { pos },
                    })
                } else {
                    IntersectionResult::NoIntersection
                }
            }
            _ => IntersectionResult::NoIntersection,
        }
    }

    fn bounding_box(&self) -> Bounds {
        Bounds::from_points(&self.vertices)
    }
}
