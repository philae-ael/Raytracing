use crate::{
    math::bounds::Bounds,
    ray::Ray,
    shape::{FullIntersectionResult, IntersectionResult, MinIntersectionResult, Shape},
};

use super::Aggregate;
pub enum ShapeListEntry {
    Shape(Box<dyn Shape>),
    List(ShapeList),
}

impl ShapeListEntry {
    pub fn as_shape(&self) -> &dyn Shape {
        match self {
            ShapeListEntry::Shape(s) => s.as_ref(),
            ShapeListEntry::List(l) => l,
        }
    }
}

#[derive(Default)]
pub struct ShapeList(pub Vec<ShapeListEntry>);
impl Aggregate for ShapeList {}

impl Shape for ShapeList {
    fn intersection_full(&self, mut ray: Ray) -> FullIntersectionResult {
        let mut res = IntersectionResult::NoIntersection;

        for entry in self.0.iter() {
            let shape = entry.as_shape();
            if ray.range().is_empty() {
                return IntersectionResult::NoIntersection;
            }

            if let IntersectionResult::Intersection(record) = shape.intersection_full(ray) {
                ray.bounds.1 = record.t;
                res = IntersectionResult::Intersection(record);
            }
        }
        res
    }

    fn intersect_bare(&self, _ray: Ray) -> MinIntersectionResult {
        todo!()
    }

    fn bounding_box(&self) -> Bounds {
        self.0
            .iter()
            .map(|x| x.as_shape().bounding_box())
            .reduce(|x, y| Bounds::from_bounds(x, y))
            .expect("Expected at least one shape")
    }
}
