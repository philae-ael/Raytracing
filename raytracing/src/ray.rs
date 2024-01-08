use std::ops::{Range, RangeInclusive};

use super::math::vec::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub bounds: (f32, f32),
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            bounds: (0.0, std::f32::INFINITY),
        }
    }
    pub fn new_with_range(origin: Vec3, direction: Vec3, range: Range<f32>) -> Self {
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

    pub fn at(&self, t: f32) -> Vec3 {
        if !self.range().contains(&t) {
            crate::utils::log_once::error_once!("a ray has been accessed out of bounds");
        }

        self.at_unchecked(t)
    }
    pub fn at_unchecked(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::Ray;

    #[test]
    fn ray() {
        let eps = 0.01;
        let ray = Ray::new(
            Vec3 {
                x: 1.,
                y: 0.,
                z: 0.,
            },
            Vec3 {
                x: -1.,
                y: 1.,
                z: 0.,
            },
        );

        assert!(ray.at(0.0).distance_squared(ray.origin) < eps);
        assert!(ray.at(1.0).distance_squared(ray.origin + ray.direction) < eps);
    }
}
