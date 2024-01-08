use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

pub fn lerp<T>(t: f64, x: T, y: T) -> T
where
    T: Add<T, Output = T> + std::cmp::PartialEq,
    f64: Mul<T, Output = T>,
{
    t * x + (1.0 - t) * y
}

pub fn clamp<T: From<f64> + PartialOrd>(x: T) -> T {
    if x > 1.0.into() {
        1.0.into()
    } else if x < 0.0.into() {
        0.0.into()
    } else {
        x
    }
}

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

#[derive(Default)]
pub struct UnitBall3RejectionMethod;
#[derive(Default)]
pub struct UnitBall3PolarMethod;

#[derive(Default)]
pub struct UnitBall3<Method = UnitBall3RejectionMethod> {
    _phantom: PhantomData<Method>,
}

impl Distribution<[f64; 3]> for UnitBall3<UnitBall3RejectionMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 3] {
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
impl Distribution<[f64; 3]> for UnitBall3<UnitBall3PolarMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 3] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f64::consts::TAU * uniform.sample(rng);
        let (sp, cp) = f64::sin_cos(phi);
        let theta = std::f64::consts::PI * uniform.sample(rng);
        let (st, ct) = f64::sin_cos(theta);
        let x = uniform.sample(rng);
        let r = x.powf(1. / 3.);
        [r * cp * st, r * sp * st, r * ct]
    }
}

pub struct UnitBall2;
impl Distribution<[f64; 2]> for UnitBall2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 2] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f64::consts::TAU * uniform.sample(rng);
        let x = uniform.sample(rng);
        let r = x.sqrt();
        let (s, c) = f64::sin_cos(phi);
        [r * c, r * s]
    }
}

pub struct UnitSphere2;
impl Distribution<[f64; 2]> for UnitSphere2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 2] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f64::consts::TAU * uniform.sample(rng);
        let (s, c) = f64::sin_cos(phi);
        [c, s]
    }
}
