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

pub fn clamp(x: f64) -> f64 {
    if x > 1.0 {
        1.0
    } else if x < 0.0 {
        0.0
    } else {
        x
    }
}

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};

#[derive(Default)]
pub struct UnitSphere3RejectionMethod;
#[derive(Default)]
pub struct UnitSphere3PolarMethod;

#[derive(Default)]
pub struct UnitSphere3<Method = UnitSphere3RejectionMethod> {
    _phantom: PhantomData<Method>,
}

impl Distribution<[f64; 3]> for UnitSphere3<UnitSphere3RejectionMethod> {
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
impl Distribution<[f64; 3]> for UnitSphere3<UnitSphere3PolarMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f64; 3] {
        let uniform = Uniform::new(0., 1.);
        let phi = std::f64::consts::TAU * uniform.sample(rng);
        let theta = std::f64::consts::PI * uniform.sample(rng);
        let x = uniform.sample(rng);
        let r = x.powf(1. / 3.);
        [
            r * f64::cos(phi) * f64::sin(theta),
            r * f64::sin(phi) * f64::sin(theta),
            r * f64::cos(theta),
        ]
    }
}
