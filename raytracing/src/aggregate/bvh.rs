use glam::Vec3;

use crate::{
    aggregate::shapelist::ShapeListEntry,
    math::bounds::Bounds,
    ray::Ray,
    shape::{FullIntersectionResult, IntersectionResult, MinIntersectionResult, Shape},
};

use super::shapelist::ShapeList;

pub struct BVH {
    bounding_box: Bounds,
    node: BVHNode,
}

pub enum BVHNode {
    Node(Box<BVH>, Box<BVH>),
    Leaf(Box<dyn Shape>),
}

impl BVH {
    pub fn from_shapelist(s: ShapeList) -> Self {
        // A Top down implementation
        let bounding_box = s.bounding_box();
        let ShapeList(mut shapes) = s;

        let n = shapes.len();
        assert!(n > 0);

        if n == 1 {
            match shapes.remove(0) {
                ShapeListEntry::Shape(shape) => {
                    return Self {
                        bounding_box: shape.bounding_box(),
                        node: BVHNode::Leaf(shape),
                    };
                }
                ShapeListEntry::List(list) => return Self::from_shapelist(list),
            }
        }

        let half = n / 2;

        // Sort by main axis
        let Vec3 { x, y, z } = bounding_box.diag();
        let main_axis: fn(Vec3) -> f32 = if x >= y && x >= z {
            |v| v.x
        } else if y >= z {
            |v| v.y
        } else {
            |v| v.z
        };

        shapes.sort_by(|a, b| {
            let a = a.as_shape().bounding_box().origin.vec();
            let b = b.as_shape().bounding_box().origin.vec();
            main_axis(a).partial_cmp(&main_axis(b)).unwrap()
        });

        // Take half of it in a node, the other half in the other node
        let second_batch = shapes.split_off(half);
        let first_batch = shapes;

        Self {
            bounding_box,
            node: BVHNode::Node(
                Box::new(Self::from_shapelist(ShapeList(first_batch))),
                Box::new(Self::from_shapelist(ShapeList(second_batch))),
            ),
        }
    }
}

impl Shape for BVH {
    fn intersection_full(&self, ray: Ray) -> FullIntersectionResult {
        if let Some(_) = self.bounding_box.ray_intersect(&ray) {
            match &self.node {
                BVHNode::Node(a, b) => {
                    if let IntersectionResult::Intersection(record) = a.intersection_full(ray) {
                        let ray2 = Ray {
                            bounds: (ray.bounds.0, record.t),
                            ..ray
                        };
                        let isect2 = b.intersection_full(ray2);
                        if isect2.is_intersection() {
                            isect2
                        } else {
                            IntersectionResult::Intersection(record)
                        }
                    } else {
                        b.intersection_full(ray)
                    }
                }
                BVHNode::Leaf(l) => l.intersection_full(ray),
            }
        } else {
            IntersectionResult::NoIntersection
        }
    }

    fn intersect_bare(&self, _ray: Ray) -> MinIntersectionResult {
        todo!()
    }

    fn bounding_box(&self) -> Bounds {
        self.bounding_box
    }
}
