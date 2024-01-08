use std::ops::Range;

use glam::Vec3;

use crate::ray::Ray;

use super::point::Point;

/// Axis Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub origin: Point,
    /// Should have all coordinates >= 0
    pub diag: Vec3,
}

impl Bounds {
    pub fn from_points(a: Point, b: Point) -> Self {
        let Vec3 {
            x: xa,
            y: ya,
            z: za,
        } = a.vec();
        let Vec3 {
            x: xb,
            y: yb,
            z: zb,
        } = b.vec();

        let origin = Point::new(f32::min(xa, xb), f32::min(ya, yb), f32::min(za, zb));
        let end = Point::new(f32::max(xa, xb), f32::max(ya, yb), f32::max(za, zb));

        Self {
            origin,
            diag: end - origin,
        }
    }

    pub fn end(&self) -> Point {
        self.origin + self.diag
    }
    pub fn volume(&self) -> f32 {
        let Vec3 { x, y, z } = self.diag;
        x * y * z
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if other.contains(self.origin) {
            Some(Bounds::from_points(self.origin, other.end()))
        } else if self.contains(other.origin) {
            Some(Bounds::from_points(other.origin, self.end()))
        } else {
            None
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        let origin = AABBPointOrder(self.origin);
        let end = AABBPointOrder(self.end());
        let point = AABBPointOrder(point);

        origin <= point && point <= end
    }

    pub fn ray_intersect(&self, ray: &Ray) -> Option<Range<f32>> {
        // R(t) = ray.origin + t*ray.dir => t*ray_dir = R(t) - ray.origin
        // We want self.origin <= R(t) <= self.end thus
        // self.origin - ray.origin <= t*ray_dir <= self.end - ray.origin
        // Solving for x y z and taking the intersection of results
        // note that we want that -x/0.0 = -infty and +x/0.0 = 0.0 for x > 0
        // for x = 0 can return NaN. Is it an issue ?

        let origin = self.origin - ray.origin;
        let end = self.origin - ray.origin;
        let ts_start = origin / ray.direction;
        let ts_end = end / ray.direction;

        let t_min = Vec3::min(ts_start, ts_end).max_element();
        let t_max = Vec3::max(ts_start, ts_end).min_element();

        assert!(!t_min.is_nan());
        assert!(!t_max.is_nan());
        assert!(self.contains(ray.at_unchecked((t_min + t_max) / 2.0)));

        if t_min > t_max {
            None
        } else {
            Some(t_min..t_max)
        }
    }
}

/// A private type that allows for custom order on points
#[derive(PartialEq)]
struct AABBPointOrder(Point);

impl PartialOrd for AABBPointOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let v = other.0.vec() - self.0.vec();

        if v.x < 0.0 && v.y < 0.0 && v.z < 0.0 {
            Some(std::cmp::Ordering::Less)
        } else if v.x > 0.0 && v.y > 0.0 && v.z > 0.0 {
            Some(std::cmp::Ordering::Greater)
        } else if v.x == 0.0 && v.y == 0.0 && v.z == 0.0 {
            Some(std::cmp::Ordering::Equal)
        } else {
            None
        }
    }
}
