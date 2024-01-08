#![allow(clippy::op_ref)]
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

use super::vec::Vec3;

/// Cfg Geometrie Berger 8.9.1 pour les maths
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion(pub [f64; 4]);

impl Quaternion {
    pub const ZERO: Self = Quaternion([0.0, 0.0, 0.0, 0.0]);
    pub const ONE: Self = Quaternion([1.0, 0.0, 0.0, 0.0]);
    pub const I: Self = Quaternion([0.0, 1.0, 0.0, 0.0]);
    pub const J: Self = Quaternion([0.0, 0.0, 1.0, 0.0]);
    pub const K: Self = Quaternion([0.0, 0.0, 0.0, 1.0]);

    pub fn inv(&self) -> Self {
        self.conjugate() / self.norm2()
    }

    pub fn from_real_pure(r: f64, v: Vec3) -> Self {
        Quaternion([r, v[0], v[1], v[2]])
    }
    pub fn real(&self) -> f64 {
        self[0]
    }

    pub fn pure(&self) -> Vec3 {
        Vec3([self[1], self[2], self[3]])
    }

    pub fn from_rotation(angle: f64, axe: Vec3) -> Self {
        let (s, c) = f64::sin_cos(angle / 2.0);
        Self::from_real_pure(c, s * axe.normalize())
    }

    pub fn conjugate(&self) -> Self {
        Self::from_real_pure(self.real(), -self.pure())
    }

    pub fn norm2(&self) -> f64 {
        (self * &self.conjugate()).real()
    }

    pub fn rotate(&self, v: Vec3) -> Vec3 {
        (self * &Self::from_real_pure(0.0, v) * self.inv()).pure()
    }

    // Returns the rotation mapping -Z to direction
    pub fn from_direction(direction: &Vec3, forward: &Vec3) -> Self {
        let direction = direction.normalize();
        let forward = forward.normalize();

        let cos = forward.dot(&direction);
        let angle = cos.acos();
        if f64::abs(cos.abs() - 1.0) <= 0.01 {
            return Self::from_rotation(angle, Vec3::Y);
        }
        let axe = forward.cross(&direction);
        Self::from_rotation(angle, axe.normalize())
    }
}

impl Index<usize> for Quaternion {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Quaternion {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl Add for &Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Self) -> Self::Output {
        Quaternion([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}
impl Sub for &Quaternion {
    type Output = Quaternion;

    fn sub(self, rhs: Self) -> Self::Output {
        Quaternion([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3],
        ])
    }
}
impl Mul for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        let x = self.0;
        let y = rhs.0;
        let a = x[0] * y[0] - x[1] * y[1] - x[2] * y[2] - x[3] * y[3];
        let b = x[0] * y[1] + x[1] * y[0] + x[2] * y[3] - x[3] * y[2];
        let c = x[0] * y[2] - x[1] * y[3] + x[2] * y[0] + x[3] * y[1];
        let d = x[0] * y[3] + x[1] * y[2] - x[2] * y[1] + x[3] * y[0];

        Quaternion([a, b, c, d])
    }
}
impl Neg for &Quaternion {
    type Output = Quaternion;

    fn neg(self) -> Self::Output {
        &Quaternion::ZERO - self
    }
}
impl Mul<&Quaternion> for f64 {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Self::Output {
        Quaternion([
            self * rhs.0[0],
            self * rhs.0[1],
            self * rhs.0[2],
            self * rhs.0[3],
        ])
    }
}
impl Div<f64> for &Quaternion {
    type Output = Quaternion;
    fn div(self, rhs: f64) -> Self::Output {
        Quaternion([
            self.0[0] / rhs,
            self.0[1] / rhs,
            self.0[2] / rhs,
            self.0[3] / rhs,
        ])
    }
}

impl Add for Quaternion {
    type Output = Quaternion;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}
impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}
impl Mul<Quaternion> for f64 {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        self * &rhs
    }
}
impl Div<f64> for Quaternion {
    type Output = Quaternion;
    fn div(self, rhs: f64) -> Self::Output {
        &self / rhs
    }
}
impl Sub for Quaternion {
    type Output = Quaternion;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
impl Neg for Quaternion {
    type Output = Quaternion;

    fn neg(self) -> Self::Output {
        Quaternion::ZERO - self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mul() {
        let one = Quaternion::ONE;
        let i = Quaternion::I;
        let j = Quaternion::J;
        let k = Quaternion::K;
        assert_eq!(one * i, i);
        assert_eq!(one * j, j);
        assert_eq!(one * k, k);
        assert_eq!(i * one, i);
        assert_eq!(j * one, j);
        assert_eq!(k * one, k);
        assert_eq!(i * j, k);
        assert_eq!(j * k, i);
        assert_eq!(k * i, j);
        assert_eq!(j * i, -k);
        assert_eq!(i * k, -j);
        assert_eq!(k * j, -i);
        assert_eq!(i * i, -one);
        assert_eq!(j * j, -one);
        assert_eq!(k * k, -one);
    }
    #[test]
    fn rotate() {
        let axe = Vec3::X;
        let angle = std::f64::consts::PI / 2.;
        let rot = Quaternion::from_rotation(angle, axe);
        assert!((rot.rotate(-Vec3::Z) - Vec3::Y).near_zero());

        let axe2 = Vec3::Z;
        let angle2 = -std::f64::consts::PI / 2.;
        let rot2 = Quaternion::from_rotation(angle2, axe2);
        let rot3 = rot2 * rot;
        assert!((rot3.rotate(-Vec3::Z) - Vec3::X).near_zero());
    }
}
