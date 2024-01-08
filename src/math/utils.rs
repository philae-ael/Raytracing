use std::marker::PhantomData;

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

use crate::material::texture::Uv;

use super::vec::Vec3;

#[derive(Default)]
pub struct UnitBall3RejectionMethod;
#[derive(Default)]
pub struct UnitBall3PolarMethod;

#[derive(Default)]
pub struct UnitBall3<Method = UnitBall3RejectionMethod> {
    _phantom: PhantomData<Method>,
}

impl Distribution<[f32; 3]> for UnitBall3<UnitBall3RejectionMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 3] {
        let uniform = Uniform::new(-1., 1.);
        let mut x1;
        let mut x2;
        let mut x3;
        loop {
            x1 = uniform.sample(rng);
            x2 = uniform.sample(rng);
            x3 = uniform.sample(rng);
            if x1 * x1 + x2 * x2 + x3 * x3 <= 1. {
                break;
            }
        }
        [x1, x2, x3]
    }
}

/// Constant time, but maybe still slower due to powf, cos, sin ?
impl Distribution<[f32; 3]> for UnitBall3<UnitBall3PolarMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 3] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f32::consts::TAU * uniform.sample(rng);
        let (sp, cp) = f32::sin_cos(phi);
        let theta = std::f32::consts::PI * uniform.sample(rng);
        let (st, ct) = f32::sin_cos(theta);
        let x = uniform.sample(rng);
        let r = x.powf(1. / 3.);
        [r * cp * st, r * sp * st, r * ct]
    }
}

pub struct UnitBall2;
impl Distribution<[f32; 2]> for UnitBall2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 2] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f32::consts::TAU * uniform.sample(rng);
        let x = uniform.sample(rng);
        let r = x.sqrt();
        let (s, c) = f32::sin_cos(phi);
        [r * c, r * s]
    }
}

pub struct UnitSphere2;
impl Distribution<[f32; 2]> for UnitSphere2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 2] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f32::consts::TAU * uniform.sample(rng);
        let (s, c) = f32::sin_cos(phi);
        [c, s]
    }
}

pub fn sphere_uv_from_direction(direction: Vec3) -> Uv {
    let h = direction.dot(Vec3::Y);
    let a = (direction - (h * Vec3::Y)).normalize();
    let u = 0.5 + f32::atan2(a.x, a.z) / std::f32::consts::TAU;
    let v = f32::acos(h) / std::f32::consts::PI;

    [u, v]
}
