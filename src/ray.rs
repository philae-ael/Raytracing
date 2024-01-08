use std::marker::PhantomData;

use super::math::vec::Vec3;

#[derive(Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    _marker: PhantomData<()> // Can't construct Ray without new
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            _marker: Default::default()
        }
    }
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}
