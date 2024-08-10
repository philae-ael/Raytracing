use core::f32;
use std::{marker::PhantomData, ops::Deref};

use glam::Vec2;
use log::trace;
use rand::{distributions::Uniform, prelude::Distribution, Rng};

use crate::material::texture::Uv;

use super::{float::FloatAsExt, vec::Vec3};

/// Samples are expected to be in [0;1(^N
pub struct Samples<const N: usize>(pub [f32; N]);
pub struct SampleND<'a>(pub &'a [f32]);
pub type Sample1D = Samples<1>;
pub type Sample2D = Samples<2>;

impl<const N: usize> Deref for Samples<N> {
    type Target = [f32; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Samplable<T, const N: usize> {
    fn sample_with(&self, samples: Samples<N>) -> T;
}

pub trait DirectionalPDF {
    fn pdf(&self, costheta: f32) -> f32;
}

#[derive(Default)]
pub struct UnitBall3RejectionMethod;
#[derive(Default)]
pub struct UniformUnitBall3PolarMethod;

#[derive(Default)]
pub struct UniformUnitBall3<Method = UnitBall3RejectionMethod> {
    _phantom: PhantomData<Method>,
}

impl Distribution<[f32; 3]> for UniformUnitBall3<UnitBall3RejectionMethod> {
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

impl Samplable<[f32; 3], 3> for UniformUnitBall3<UniformUnitBall3PolarMethod> {
    fn sample_with(&self, samples: Samples<3>) -> [f32; 3] {
        let phi = std::f32::consts::TAU * samples[0];
        let (sp, cp) = f32::sin_cos(phi);
        let theta = std::f32::consts::PI * samples[1];
        let (st, ct) = f32::sin_cos(theta);
        let x = samples[2];
        let r = x.powf(1. / 3.);
        [r * cp * st, r * sp * st, r * ct]
    }
}

/// Constant time, but maybe still slower due to powf, cos, sin ?
impl Distribution<[f32; 3]> for UniformUnitBall3<UniformUnitBall3PolarMethod> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 3] {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([
            uniform.sample(rng),
            uniform.sample(rng),
            uniform.sample(rng),
        ]))
    }
}

pub struct UniformUnitBall2;
impl Samplable<[f32; 2], 2> for UniformUnitBall2 {
    fn sample_with(&self, samples: Samples<2>) -> [f32; 2] {
        let phi = std::f32::consts::TAU * samples[0];
        let x = samples[1];
        let r = x.sqrt();
        let (s, c) = f32::sin_cos(phi);
        [r * c, r * s]
    }
}

impl Distribution<[f32; 2]> for UniformUnitBall2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 2] {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([uniform.sample(rng), uniform.sample(rng)]))
    }
}

pub struct UniformUnitSphere2;
impl Samplable<[f32; 2], 1> for UniformUnitSphere2 {
    fn sample_with(&self, samples: Samples<1>) -> [f32; 2] {
        let phi = std::f32::consts::TAU * samples[0];
        let (s, c) = f32::sin_cos(phi);
        [c, s]
    }
}

impl Distribution<[f32; 2]> for UniformUnitSphere2 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [f32; 2] {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([uniform.sample(rng)]))
    }
}

pub struct UniformUnitSphere3;
impl Samplable<Vec3, 2> for UniformUnitSphere3 {
    fn sample_with(&self, samples: Samples<2>) -> Vec3 {
        let phi = std::f32::consts::TAU * samples[0];
        let theta = std::f32::consts::PI * samples[1];

        let (sphi, cphi) = f32::sin_cos(phi);
        let (stheta, ctheta) = f32::sin_cos(theta);
        [cphi * stheta, sphi * stheta, ctheta].into()
    }
}

impl Distribution<Vec3> for UniformUnitSphere3 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([uniform.sample(rng), uniform.sample(rng)]))
    }
}

pub fn sphere_uv_from_direction(direction: Vec3) -> Uv {
    let h = direction.dot(Vec3::Y);
    let a = (direction - (h * Vec3::Y)).normalize();
    let u = 0.5 + f32::atan2(a.x, a.z) / std::f32::consts::TAU;
    let v = f32::acos(h) / std::f32::consts::PI;

    [u, v]
}

pub struct UniformHemisphere3;

impl Samplable<Vec3, 2> for UniformHemisphere3 {
    fn sample_with(&self, samples: Samples<2>) -> Vec3 {
        let z = samples[0];
        let r = f32::sqrt(1.0 - z * z);
        let (s, c) = f32::sin_cos(std::f32::consts::TAU * samples[1]);

        Vec3 {
            x: r * c,
            y: r * s,
            z,
        }
    }
}
impl DirectionalPDF for UniformHemisphere3 {
    fn pdf(&self, _costheta: f32) -> f32 {
        f32::consts::FRAC_1_PI
    }
}

impl Distribution<Vec3> for UniformHemisphere3 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([uniform.sample(rng), uniform.sample(rng)]))
    }
}

pub struct CosineHemisphere3;
impl Samplable<Vec3, 2> for CosineHemisphere3 {
    fn sample_with(&self, samples: Samples<2>) -> Vec3 {
        let p = UniformUnitBall2.sample_with(samples);
        let z = f32::sqrt(1.0 - p[0] * p[0] - p[1] * p[1]);

        Vec3 {
            x: p[0],
            y: p[1],
            z,
        }
    }
}

impl DirectionalPDF for CosineHemisphere3 {
    fn pdf(&self, costheta: f32) -> f32 {
        costheta * f32::consts::FRAC_1_PI
    }
}

impl Distribution<Vec3> for CosineHemisphere3 {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        let uniform = Uniform::new(0., 1.);
        self.sample_with(Samples([uniform.sample(rng), uniform.sample(rng)]))
    }
}

pub struct IsotropicTrowbridgeReitzDistribution {
    pub alpha: f32,
}

impl IsotropicTrowbridgeReitzDistribution {
    pub fn is_smooth(&self) -> bool {
        self.alpha < 1e-8
    }
    pub fn dw(&self, w: Vec3, wm: Vec3) -> f32 {
        self.g1(w) / w.z.abs() * self.d(wm) * w.dot(wm).abs()
    }
    pub fn d(&self, wm: Vec3) -> f32 {
        let cos2theta = wm.z * wm.z;
        let sin2theta = 1.0 - cos2theta;
        let tan2theta = sin2theta / cos2theta;

        1.0 / (f32::consts::PI * cos2theta.powi(2) * (self.alpha + tan2theta).powi(2))
    }
    pub fn g(&self, w0: Vec3, w1: Vec3) -> f32 {
        1.0 / (1.0 + self.lambda(w0) + self.lambda(w1))
    }
    pub fn g1(&self, w: Vec3) -> f32 {
        1.0 / (1.0 + self.lambda(w))
    }
    pub fn lambda(&self, w: Vec3) -> f32 {
        let cos2theta = w.z * w.z;
        let sin2theta = 1.0 - cos2theta;
        let tan2theta = sin2theta / cos2theta;

        (f32::sqrt(1.0 + self.alpha.powi(2) * tan2theta)) / 2.0
    }

    pub fn pdf(&self, w: Vec3, wm: Vec3) -> f32 {
        self.dw(w, wm)
    }

    pub fn sample_wm(&self, w: Vec3, samples: Samples<2>) -> Vec3 {
        let w = Vec3::new(self.alpha * w.x, self.alpha * w.y, w.z).normalize();
        let w = if w.z >= 0.0 { w } else { -w };
        trace!("{w:?}");

        let t1 = if w.z < 0.9999 {
            Vec3::Z.cross(w).normalize()
        } else {
            Vec3::X
        };
        let t2 = w.cross(t1);

        let p = {
            let mut p = UniformUnitBall2.sample_with(samples);

            let h = f32::sqrt(1.0 - p[0].powi(2));
            p[1] = ((1.0 + w.z) / 2.0).lerp(h, p[1]);
            Vec2::from_array(p)
        };

        let pz = f32::sqrt(f32::max(0.0, 1.0 - p.length_squared()));
        let nh = p.x * t1 + p.y * t2 + pz * w;

        // Normal -> a and not 1/a
        Vec3::new(self.alpha * nh.x, self.alpha * nh.y, f32::max(1e-6, nh.z)).normalize()
    }
}
