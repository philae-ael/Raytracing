use std::marker::PhantomData;

use super::math::vec::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    _marker: PhantomData<()>, // Can't construct Ray without new
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            _marker: Default::default(),
        }
    }
    pub fn at(&self, t: f32) -> Vec3 {
        //assert!(t >= 0.0, "a ray can only be accessed at positive time");
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
