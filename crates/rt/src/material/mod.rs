mod dielectric;
mod diffuse;
mod emit;
mod gooch;
mod metal;
mod mix;
mod phong;
pub mod texture;

use core::f32;

use bitflags::bitflags;
pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use emit::Emit;
pub use gooch::Gooch;
pub use metal::Metal;
pub use mix::MixMaterial;

use glam::Vec3;
use rand::{distributions::Uniform, prelude::Distribution};

use crate::{
    color::{linear::BLACK, Rgb},
    math::{
        distributions::{
            CosineHemisphere3, DirectionalPDF, Samplable, Sample1D, Sample2D, Samples,
        },
        point::Point,
        transform::Frame,
    },
    ray::Ray,
    shape::local_info,
};

pub trait Material: Sync + Send {
    fn scatter(&self, ray: Ray, record: &local_info::Full, rng: &mut crate::Rng) -> Scattered;

    fn transmission(&self) -> Option<(f32, Vec3)> {
        None
    }
    fn reflection(&self) -> Option<Vec3> {
        None
    }

    fn diffuse(&self) -> Option<Vec3> {
        None
    }

    fn emissive(&self) -> Option<Vec3> {
        None
    }
}

pub struct BSDFMaterial<B: BxDF> {
    pub bxdf: B,
}

impl<B: BxDF + Sync + Send> Material for BSDFMaterial<B> {
    fn scatter(&self, ray: Ray, record: &local_info::Full, rng: &mut crate::Rng) -> Scattered {
        let bsdf = BSDF::new(record.normal, &self.bxdf);

        let uniform = Uniform::new(0.0, 1.0);
        let s = bsdf.sample_f(
            ray.direction,
            Samples([uniform.sample(rng), uniform.sample(rng)]),
            Samples([uniform.sample(rng)]),
        );

        match s {
            Some(s) => Scattered {
                albedo: s.sampled_f,
                ray_out: Some(Ray::new(record.pos, s.wi)),
            },
            None => Scattered {
                albedo: BLACK,
                ray_out: None,
            },
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BxDFFlags : u8 {
        const Reflection = 0b00000001;
        const Diffusion = 0b00000010;
    }
}

pub struct BxDFSample {
    pub wi: Vec3,
    pub sampled_f: Rgb,
    pub pdf: f32,
}

pub trait BxDF {
    const FLAGS: BxDFFlags;
    // Note that the output is a mere float: The output of the bxdf is supposed to not depend on
    // the wavelength
    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb;
    fn pdf(&self, wo: Vec3, wi: Vec3) -> f32;

    /// uvw is used for sampling and should be sampled in [0;1)^2
    fn sample_f(&self, wo: Vec3, uv: Sample2D, w: Sample1D) -> Option<BxDFSample>;
}

pub struct BSDF<'a, I: BxDF> {
    inner: &'a I,
    frame: Frame,
}

impl<'a, I: BxDF> BSDF<'a, I> {
    /// Normal should be normalized
    pub fn new(normal: Vec3, bxdf: &'a I) -> Self {
        Self {
            inner: bxdf,
            frame: Frame::new(normal),
        }
    }
}

impl<I: BxDF> BxDF for BSDF<'_, I> {
    const FLAGS: BxDFFlags = I::FLAGS;

    fn sample_f(&self, wo: Vec3, uv: Sample2D, w: Sample1D) -> Option<BxDFSample> {
        self.inner
            .sample_f(self.frame.to_local(wo), uv, w)
            .map(|x| BxDFSample {
                wi: self.frame.from_local(x.wi),
                ..x
            })
    }

    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb {
        self.inner
            .f(self.frame.to_local(wo), self.frame.to_local(wi))
    }
    fn pdf(&self, wo: Vec3, wi: Vec3) -> f32 {
        self.inner
            .pdf(self.frame.to_local(wo), self.frame.to_local(wi))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffuseBxDF {
    pub albedo: Rgb,
}

#[inline]
fn same_hemishpere(v: Vec3, w: Vec3) -> bool {
    v.z * w.z > 0.0
}

impl BxDF for DiffuseBxDF {
    const FLAGS: BxDFFlags = BxDFFlags::Diffusion;

    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb {
        if !same_hemishpere(wo, wi) {
            return Rgb::default();
        }
        f32::consts::FRAC_1_PI * self.albedo
    }

    fn pdf(&self, _wo: Vec3, wi: Vec3) -> f32 {
        CosineHemisphere3.pdf(wi.z.abs())
    }

    fn sample_f(&self, wo: Vec3, uv: Sample2D, _w: Sample1D) -> Option<BxDFSample> {
        let mut wi = CosineHemisphere3.sample_with(uv);
        wi.z = wi.z.copysign(wo.z);

        let pdf = CosineHemisphere3.pdf(wi.z.abs());

        Some(BxDFSample {
            wi,
            sampled_f: f32::consts::FRAC_1_PI * self.albedo,
            pdf,
        })
    }
}

pub struct Scattered {
    pub albedo: Rgb,
    pub ray_out: Option<Ray>,
}

pub struct MaterialDescriptor {
    pub label: Option<String>,
    pub material: Box<dyn Material>,
}

impl std::fmt::Debug for MaterialDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialDescriptor")
            .field("label", &self.label)
            .field("material", &"<material>")
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct LightDescriptor {
    pub label: Option<String>,
    pub light_pos: Point,
}

#[derive(Debug, Clone, Copy)]
pub struct MaterialId(pub usize);
