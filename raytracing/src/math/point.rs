use std::ops::{Add, Sub};

use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub Vec3);

impl Point {
    pub const ORIGIN: Point = Point(Vec3::ZERO);
    pub fn vec(self) -> Vec3 {
        self.0
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vec3::new(x, y, z))
    }
}

impl Add<Vec3> for Point {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Point(self.vec() + rhs)
    }
}

impl Sub<Vec3> for Point {
    type Output = Self;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Point(self.vec() - rhs)
    }
}

/// We can sub two points but not add them
impl Sub for Point {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        self.vec() - rhs.vec()
    }
}
