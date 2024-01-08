use std::ops;

use glam::Vec3;

use crate::{
    hit::{Hit, HitRecord, Hittable},
    material::MaterialId,
    ray::Ray,
};

pub struct HitPoint {
    pub t: f32,
    pub hit_point: Vec3,
    pub normal: Vec3,
}
pub trait ImplicitSolver {
    fn solve<F: Fn(Vec3) -> f32>(&self, f: F, ray: Ray, range: ops::Range<f32>)
        -> Option<HitPoint>;
}

pub trait ImplicitSurface {
    fn impl_f(&self, p: Vec3) -> f32;
    fn material(&self) -> MaterialId;
}

pub struct NewtonSolver {
    pub eps: f32,
    pub max_step: usize,
}

impl ImplicitSolver for NewtonSolver {
    fn solve<F: Fn(Vec3) -> f32>(
        &self,
        f: F,
        ray: Ray,
        range: ops::Range<f32>,
    ) -> Option<HitPoint> {
        let mut t = range.start;
        let mut steps = 0;
        let f_along_ray = |t| f(ray.at(t));
        loop {
            let ft = f_along_ray(t);
            if ft.abs() < self.eps {
                break;
            } else if steps >= self.max_step {
                //log::info!("No Hit {ft}");
                return None;
            }

            steps += 1;

            let dft = (f_along_ray(t + self.eps) - f_along_ray(t)) / self.eps;

            if dft.abs() < self.eps {
                return None;
            }

            let new_t = t - ft / dft;
            t = new_t.min(range.end).max(range.start);
        }

        let x = ray.at(t);
        let normal = Vec3::new(
            (f(x + self.eps * Vec3::X) - f(x)) / self.eps,
            (f(x + self.eps * Vec3::Y) - f(x)) / self.eps,
            (f(x + self.eps * Vec3::Z) - f(x)) / self.eps,
        );
        Some(HitPoint {
            t,
            hit_point: x,
            normal: normal.normalize(),
        })
    }
}

pub struct HittableImplicitSurface<Surf: ImplicitSurface, Solv: ImplicitSolver> {
    pub surf: Surf,
    pub solv: Solv,
}

impl<Surf: ImplicitSurface, Solv: ImplicitSolver> Hittable for HittableImplicitSurface<Surf, Solv> {
    fn hit(&self, ray: Ray, range: ops::Range<f32>) -> Hit {
        if let Some(HitPoint {
            t,
            hit_point,
            normal,
        }) = self.solv.solve(|x| self.surf.impl_f(x), ray, range)
        {
            Hit::Hit(HitRecord {
                hit_point,
                normal,
                t,
                material: self.surf.material(),
                uv: [0.0, 0.0],
            })
        } else {
            Hit::NoHit
        }
    }
}

pub struct Sphere {
    pub radius: f32,
    pub origin: Vec3,
    pub material: MaterialId,
}


impl ImplicitSurface for Sphere {
    fn impl_f(&self, p: Vec3) -> f32 {
        self.origin.distance_squared(p) - self.radius * self.radius
    }

    fn material(&self) -> MaterialId {
        self.material
    }
}
pub struct Cube {
    pub size: f32,
    pub origin: Vec3,
    pub material: MaterialId,
}

impl ImplicitSurface for Cube {
    fn impl_f(&self, p: Vec3) -> f32 {
        self.size/2.0 - (p - self.origin).abs().max_element()
    }

    fn material(&self) -> MaterialId {
        self.material
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{hit::Hittable, material::MaterialId, ray::Ray};

    use super::{HittableImplicitSurface, ImplicitSurface, NewtonSolver, Sphere};

    #[test]
    fn sphere_impl_surf() {
        let eps = 0.01;
        let sphere = Sphere {
            radius: 1.,
            origin: Vec3::ZERO,
            material: MaterialId(0),
        };

        assert!(
            sphere
                .impl_f(Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.0
                })
                .abs()
                > eps
        );
        assert!(
            sphere
                .impl_f(Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0
                })
                .abs()
                < eps
        );
    }

    #[test]
    fn sphere_hit() {
        let sphere = HittableImplicitSurface {
            surf: Sphere {
                radius: 0.5,
                origin: Vec3::X,
                material: MaterialId(0),
            },
            solv: NewtonSolver {
                eps: 0.00001,
                max_step: 10,
            },
        };

        let hit = sphere.hit(Ray::new(Vec3::ZERO, Vec3::X), 0.0..std::f32::INFINITY);
        match hit {
            crate::hit::Hit::Hit(h) => {
                assert!(h
                    .hit_point
                    .distance_squared(Vec3::new(0.5, 0., 0.)) < sphere.solv.eps);
            }
            crate::hit::Hit::NoHit => panic!("{hit:?}"),
        }
    }
}
