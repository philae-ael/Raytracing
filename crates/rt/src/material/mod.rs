pub mod texture;

use std::ops::Deref;

use bitflags::bitflags;
use glam::Vec3;

use crate::{
    color::{
        linear::{BLACK, WHITE},
        Rgb,
    },
    math::{
        distributions::{CosineHemisphere3, DirectionalPDF, Samplable, Sample1D, Sample2D},
        point::Point,
        transform::Frame,
        vec::Vec3Ext,
    },
    ray::Ray,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BxDFFlags : u8 {
        const Reflection = 0b00000001;
        const Diffusion = 0b00000010;
        const Transmission = 0b00000100;
        const Specular= 0b00001000;
    }
}

pub struct BxDFSample {
    pub wi: Vec3,
    pub f: Rgb,
    pub pdf: f32,
}

pub trait BxDF {
    fn flags(&self) -> BxDFFlags;
    // Note that the output is a mere float: The output of the bxdf is supposed to not depend on
    // the wavelength
    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb;
    fn pdf(&self, wo: Vec3, wi: Vec3) -> f32;

    /// uvw is used for sampling and should be sampled in [0;1)^2
    fn sample_f(&self, wo: Vec3, uv: Sample2D, w: Sample1D) -> Option<BxDFSample>;

    // NOTE: This should not be here!
    fn le(&self) -> Rgb {
        BLACK
    }
}

pub struct BSDF<'a, I: BxDF + ?Sized> {
    inner: &'a I,
    frame: Frame,
}

impl<'a, I: BxDF + ?Sized> BSDF<'a, I> {
    /// Normal should be normalized
    pub fn new(normal: Vec3, bxdf: &'a I) -> Self {
        Self {
            inner: bxdf,
            frame: Frame::new(normal),
        }
    }
}

impl<I: BxDF + ?Sized> BSDF<'_, I> {
    pub fn flags(&self) -> BxDFFlags {
        self.inner.flags()
    }

    pub fn sample_f(&self, wo: Vec3, uv: Sample2D, w: Sample1D) -> Option<BxDFSample> {
        self.inner
            .sample_f(self.frame.to_local(wo), uv, w)
            .map(|x| BxDFSample {
                wi: self.frame.from_local(x.wi),
                ..x
            })
    }

    pub fn f(&self, wo: Vec3, wi: Vec3) -> Rgb {
        self.inner
            .f(self.frame.to_local(wo), self.frame.to_local(wi))
    }
    pub fn pdf(&self, wo: Vec3, wi: Vec3) -> f32 {
        self.inner
            .pdf(self.frame.to_local(wo), self.frame.to_local(wi))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DiffuseBxDF {
    pub albedo: Rgb,
}

impl BxDF for DiffuseBxDF {
    fn flags(&self) -> BxDFFlags {
        BxDFFlags::Diffusion
    }

    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb {
        if !wo.same_hemishpere(wi) {
            return Rgb::default();
        }
        core::f32::consts::FRAC_1_PI * self.albedo
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
            f: core::f32::consts::FRAC_1_PI * self.albedo,
            pdf,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DielectricBxDF {
    pub ior: f32,
}

fn fresnel_dielectric(cosi: f32, ior: f32) -> f32 {
    let (cosi, ior) = if cosi >= 0.0 {
        (cosi, ior)
    } else {
        (-cosi, 1.0 / ior)
    };

    let costsq = 1.0 - (1.0 - cosi) * cosi / ior / ior;
    if costsq < 0.0 {
        return 1.0; // complete reflection
    }
    let cost = f32::sqrt(costsq);
    let r_parl = (ior * cosi - cost) / (ior * cosi + cost);
    let r_perp = (cosi - ior * cost) / (cosi + ior * cost);
    0.5 * (r_parl + r_perp)
}

impl BxDF for DielectricBxDF {
    fn flags(&self) -> BxDFFlags {
        let f = if self.ior == 1.0 {
            BxDFFlags::Transmission
        } else {
            BxDFFlags::empty()
        };

        f | BxDFFlags::Reflection | BxDFFlags::Specular
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Rgb {
        BLACK
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f32 {
        0.0
    }

    fn sample_f(&self, wo: Vec3, _uv: Sample2D, w: Sample1D) -> Option<BxDFSample> {
        let r = fresnel_dielectric(wo.z, self.ior);
        let t = 1.0 - r;

        if w[0] <= r / (r + t) {
            // perfect reflection
            let wi = Vec3::new(-wo.x, -wo.y, wo.z);
            Some(BxDFSample {
                wi,
                f: (r / wi.z.abs()) * WHITE,
                pdf: r / (r + t),
            })
        } else {
            // perfect transmission (with refraction)
            let wi = wo.refract(Vec3::Z, self.ior)?;
            Some(BxDFSample {
                wi,
                f: (t / wi.z.abs()) * WHITE,
                pdf: t / (r + t),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ThinDielectricBxDF {
    pub ior: f32,
}

impl BxDF for ThinDielectricBxDF {
    fn flags(&self) -> BxDFFlags {
        let f = if self.ior == 1.0 {
            BxDFFlags::Transmission
        } else {
            BxDFFlags::empty()
        };

        f | BxDFFlags::Reflection | BxDFFlags::Specular
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Rgb {
        BLACK
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f32 {
        0.0
    }

    fn sample_f(&self, wo: Vec3, _uv: Sample2D, w: Sample1D) -> Option<BxDFSample> {
        let r = fresnel_dielectric(wo.z, self.ior);
        let t = 1.0 - r;
        let (r, t) = if r >= 1.0 {
            (1.0, 0.0)
        } else {
            let r = r + t * t * r / (1.0 - r * r);
            (r, 1. - r)
        };

        if w[0] <= r / (r + t) {
            // perfect reflection
            let wi = Vec3::new(-wo.x, -wo.y, wo.z);
            Some(BxDFSample {
                wi,
                f: (r / wi.z.abs()) * WHITE,
                pdf: r / (r + t),
            })
        } else {
            // perfect transmission (with refraction)
            let wi = -wo;
            Some(BxDFSample {
                wi,
                f: (t / wi.z.abs()) * WHITE,
                pdf: t / (r + t),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EmitBxDF {
    pub le: Rgb,
}

impl BxDF for EmitBxDF {
    fn flags(&self) -> BxDFFlags {
        BxDFFlags::empty()
    }

    fn f(&self, _wo: Vec3, _wi: Vec3) -> Rgb {
        BLACK
    }

    fn pdf(&self, _wo: Vec3, _wi: Vec3) -> f32 {
        0.0
    }

    fn sample_f(&self, _wo: Vec3, _uv: Sample2D, _w: Sample1D) -> Option<BxDFSample> {
        None
    }

    fn le(&self) -> Rgb {
        self.le
    }
}

pub struct Scattered {
    pub albedo: Rgb,
    pub ray_out: Option<Ray>,
}

pub struct MaterialDescriptor {
    pub label: Option<String>,
    pub material: Box<dyn BxDF + Send + Sync>,
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

impl Deref for MaterialId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
