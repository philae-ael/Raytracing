use std::ops::{Range, RangeInclusive};

use crate::math::point::Point;

use super::math::vec::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vec3,
    pub bounds: (f32, f32),
}

impl Ray {
    pub fn new(origin: Point, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            bounds: (0.0, f32::INFINITY),
        }
    }
    pub fn new_with_range(origin: Point, direction: Vec3, range: Range<f32>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            bounds: (range.start, range.end),
        }
    }

    pub fn range(&self) -> RangeInclusive<f32> {
        self.bounds.0..=self.bounds.1
    }

    pub fn bounds_from_range(&mut self, range: RangeInclusive<f32>) {
        self.bounds = (*range.start(), *range.end())
    }

    pub fn at(&self, t: f32) -> Point {
        if !self.range().contains(&t) {
            crate::utils::log_once::error_once!("a ray has been accessed out of bounds");
        }

        self.at_unchecked(t)
    }
    pub fn at_unchecked(&self, t: f32) -> Point {
        self.origin + t * self.direction
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::math::point::Point;

    use super::Ray;

    #[test]
    fn ray() {
        let eps = 0.01;
        let ray = Ray::new(Point::new(1., 0., 0.), Vec3::new(-1., 1., 0.));

        assert!(ray.at(0.0).vec().distance_squared(ray.origin.vec()) < eps);
        assert!(
            ray.at(1.0)
                .vec()
                .distance_squared(ray.origin.vec() + ray.direction)
                < eps
        );
    }
}
