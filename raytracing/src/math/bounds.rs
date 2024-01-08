use std::ops::Range;

use glam::Vec3;

use crate::ray::Ray;

use super::point::Point;

/// Axis Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub origin: Point,
    pub end: Point,
}

impl Bounds {
    pub fn new(origin: Point, end: Point) -> Self {
        assert!(AABBPointOrder(origin) <= AABBPointOrder(end));

        Self { origin, end }
    }
    pub fn from_points(ps: &[Point]) -> Self {
        let origin = ps
            .iter()
            .copied()
            .reduce(|Point(x), Point(y)| Point(x.min(y)))
            .expect("Expected at least one point");
        let end = ps
            .iter()
            .copied()
            .reduce(|Point(x), Point(y)| Point(x.max(y)))
            .expect("Expected at least one point");

        Self { origin, end }
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if other.contains(self.origin) {
            Some(Bounds {
                origin: self.origin,
                end: other.end,
            })
        } else if self.contains(other.origin) {
            Some(Bounds {
                origin: other.origin,
                end: self.end,
            })
        } else {
            None
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        let origin = AABBPointOrder(self.origin);
        let end = AABBPointOrder(self.end);
        let point = AABBPointOrder(point);

        origin <= point && point <= end
    }

    pub fn ray_intersect(&self, ray: &Ray) -> Option<Range<f32>> {
        // R(t) = ray.origin + t*ray.dir => t*ray_dir = R(t) - ray.origin
        // We want self.origin <= R(t) <= self.end thus
        // self.origin - ray.origin <= t*ray_dir <= self.end - ray.origin
        // Solving for x y z and taking the intersection of results
        // note that we want that -x/0.0 = -infty and +x/0.0 = 0.0 for x > 0
        // for x = 0 can return NaN. Is it an issue ? YES

        let origin = self.origin - ray.origin;
        let end = self.end - ray.origin;
        let ts_start = origin / ray.direction;
        let ts_end = end / ray.direction;

        let nan_to_neg_infinity: fn(Vec3) -> Vec3 = |v| {
            Vec3::from_array(
                v.to_array()
                    .map(|x| if x.is_nan() { f32::INFINITY } else { x }),
            )
        };
        let nan_to_infinity: fn(Vec3) -> Vec3 = |v| {
            Vec3::from_array(
                v.to_array()
                    .map(|x| if x.is_nan() { f32::INFINITY } else { x }),
            )
        };

        let t_min =
            Vec3::min(nan_to_neg_infinity(ts_start), nan_to_neg_infinity(ts_end)).max_element();
        let t_max = Vec3::max(nan_to_infinity(ts_start), nan_to_infinity(ts_end)).min_element();

        let (ray_min, ray_max) = ray.bounds;
        let t_min = f32::max(t_min, ray_min);
        let t_max = f32::min(t_max, ray_max);

        assert!(!t_min.is_nan());
        assert!(!t_max.is_nan());

        if t_min > t_max {
            None
        } else {
            Some(t_min..t_max)
        }
    }

    pub fn from_bounds(x: Bounds, y: Bounds) -> Bounds {
        Self {
            origin: Point(x.origin.vec().min(y.origin.vec())),
            end: Point(x.end.vec().max(y.end.vec())),
        }
    }
    pub fn diag(&self) -> Vec3 {
        self.end - self.origin
    }
}

/// A private type that allows for custom order on points
#[derive(PartialEq)]
struct AABBPointOrder(Point);

impl PartialOrd for AABBPointOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let v = self.0.vec() - other.0.vec();

        if v.x == 0.0 && v.y == 0.0 && v.z == 0.0 {
            Some(std::cmp::Ordering::Equal)
        } else if v.x <= 0.0 && v.y <= 0.0 && v.z <= 0.0 {
            Some(std::cmp::Ordering::Less)
        } else if v.x >= 0.0 && v.y >= 0.0 && v.z >= 0.0 {
            Some(std::cmp::Ordering::Greater)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{math::point::Point, ray::Ray};

    use super::Bounds;

    #[test]
    fn contains() {
        let b = Bounds::from_points(&[Point::new(-1.0, 0.0, 0.0), Point::new(1.0, 1.0, -1.0)]);
        let ray = Ray::new(Point::ORIGIN, -Vec3::Z);

        assert!(b.ray_intersect(&ray).unwrap().start.abs() < 0.01);
        assert!((b.ray_intersect(&ray).unwrap().end - 1.0).abs() < 0.01);
    }
}
