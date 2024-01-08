use glam::{Quat, Vec3};

use super::point::Point;

/// Represents a transformation as translation + scale + rot
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rot: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

pub trait Transformer<T> {
    fn apply(&self, v: T) -> T;
}

impl Transform {
    const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        scale: Vec3::ONE,
        rot: Quat::IDENTITY,
    };
}

impl Transformer<Vec3> for Transform {
    /// Apply rotation then scale but not translation !
    fn apply(&self, v: Vec3) -> Vec3 {
        let rotated = self.rot.mul_vec3(v);
        let rotated_scaled = self.scale * rotated;
        rotated_scaled
    }
}

impl Transformer<Point> for Transform {
    /// Apply rotation then scale then translation
    fn apply(&self, v: Point) -> Point {
        let rotated = self.rot.mul_vec3(v.vec());
        let rotated_scaled = self.scale * rotated;
        Point(rotated_scaled) + self.translation
    }
}
