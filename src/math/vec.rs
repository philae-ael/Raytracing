#![allow(clippy::op_ref)]

use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vec3(pub [f64; 3]);

pub type Normal = Vec3;
pub type Point = Vec3;
pub type Direction = Vec3;

impl Vec3 {
    pub const X: Self = Self([1.0, 0.0, 0.0]);
    pub const Y: Self = Self([0.0, 1.0, 0.0]);
    pub const Z: Self = Self([0.0, 0.0, 1.0]);
    pub const XY: Self = Self([1.0, 1.0, 0.0]);
    pub const XZ: Self = Self([1.0, 0.0, 1.0]);
    pub const YZ: Self = Self([0.0, 1.0, 1.0]);
    pub const YX: Self = Self([1.0, 1.0, 0.0]);
    pub const ZX: Self = Self([1.0, 0.0, 1.0]);
    pub const ZY: Self = Self([0.0, 1.0, 1.0]);
    pub const XYZ: Self = Self([1.0, 1.0, 1.0]);
    pub const ZERO: Self = Self([0.0, 0.0, 0.0]);
    pub const ONES: Self = Self([1.0, 1.0, 1.0]);

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }
    pub fn dot(&self, rhs: &Self) -> f64 {
        self.0[0] * rhs.0[0] + self.0[1] * rhs.0[1] + self.0[2] * rhs.0[2]
    }
    pub fn cross(&self, rhs: &Self) -> Vec3 {
        Vec3([
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        ])
    }
    pub fn length_squared(&self) -> f64 {
        Self::dot(self, self)
    }
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }
    pub fn normalize(&self) -> Vec3 {
        self / self.length()
    }

    pub fn x(&self) -> f64 {
        self.0[0]
    }
    pub fn y(&self) -> f64 {
        self.0[1]
    }
    pub fn z(&self) -> f64 {
        self.0[2]
    }
    pub fn near_zero(&self) -> bool {
        self.length_squared() < 1e-4
    }
    pub fn reflect(&self, normal: &Vec3) -> Vec3 {
        self - &(2.0 * self.dot(normal) * normal)
    }
    pub fn refract(&self, normal: &Vec3, ior: f64) -> Vec3 {
        let cos_theta = -self.dot(normal);
        let refracted_perp = ior * (self + &(cos_theta*normal));
        let refracted_par = - f64::sqrt(1.0 - refracted_perp.length_squared())*normal;
        refracted_perp + refracted_par
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Add for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}
impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}
impl Mul for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}
impl Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        &Vec3::ZERO - self
    }
}
impl Mul<&Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: &Vec3) -> Self::Output {
        Vec3([self * rhs.0[0], self * rhs.0[1], self * rhs.0[2]])
    }
}
impl Div<f64> for &Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}
impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self * &rhs
    }
}
impl Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        &self / rhs
    }
}
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::ZERO - self
    }
}

impl From<f64> for Vec3 {
    fn from(x: f64) -> Self {
        Vec3([x, x, x])
    }
}
