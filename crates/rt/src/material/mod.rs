pub mod texture;

use std::ops::Deref;

use bitflags::bitflags;
use glam::Vec3;
use log::trace;

use crate::{
    color::{
        linear::{BLACK, WHITE},
        Rgb,
    },
    math::{
        distributions::{self, CosineHemisphere3, DirectionalPDF, Samplable, Sample1D, Sample2D},
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

#[derive(Debug)]
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
        let wo_local = self.frame.to_local(wo);
        if wo_local.z == 0.0 {
            return None;
        };

        self.inner.sample_f(wo_local, uv, w).map(|x| BxDFSample {
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
    pub roughness: f32,
}

fn fresnel_dielectric(cosi: f32, ior: f32) -> f32 {
    let (cosi, ior) = if cosi >= 0.0 {
        (cosi, ior)
    } else {
        (-cosi, 1.0 / ior)
    };

    let cost2 = 1.0 - (1.0 - cosi * cosi) / ior / ior;
    if cost2 < 0.0 {
        return 1.0; // complete reflection
    }
    let cost = f32::sqrt(cost2);
    let r_parl = (ior * cosi - cost) / (ior * cosi + cost);
    let r_perp = (cosi - ior * cost) / (cosi + ior * cost);
    0.5 * (r_parl.powi(2) + r_perp.powi(2))
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

    fn f(&self, wo: Vec3, wi: Vec3) -> Rgb {
        let distrib = distributions::IsotropicTrowbridgeReitzDistribution {
            alpha: self.roughness,
        };
        if self.ior == 1.0 || distrib.is_smooth() {
            return BLACK;
        }

        let cosi = wi.z;
        let coso = wo.z;
        let reflect = cosi * coso > 0.0;
        let ior = if reflect || coso > 0.0 {
            self.ior
        } else {
            1.0 / self.ior
        };
        let wm = {
            let wm = wi * ior + wo;
            if cosi == 0.0 || coso == 0.0 || wm.length_squared() == 0.0 {
                return BLACK;
            };
            wm.z.signum() * wm.normalize()
        };

        if wi.dot(wm) * cosi < 0.0 || wo.dot(wm) * coso < 0.0 {
            return BLACK;
        }

        let r = fresnel_dielectric(wo.dot(wm), ior);
        let t = 1.0 - r;
        if reflect {
            distrib.d(wm) * distrib.g(wo, wi) * r / f32::abs(4. * cosi * coso) * WHITE
        } else {
            let denom = (wi.dot(wm) + wo.dot(wm) / ior).powi(2) * cosi * coso;
            distrib.d(wm)
                * t
                * distrib.g(wo, wi)
                * f32::abs(wi.dot(wm) * wo.dot(wm) / denom)
                * WHITE
        }
    }

    fn pdf(&self, wo: Vec3, wi: Vec3) -> f32 {
        let distrib = distributions::IsotropicTrowbridgeReitzDistribution {
            alpha: self.roughness,
        };
        if self.ior == 1.0 || distrib.is_smooth() {
            return 0.0;
        }

        let cosi = wi.z;
        let coso = wo.z;
        let reflect = cosi * cosi > 0.0;
        let ior = if reflect { self.ior } else { 1.0 / self.ior };
        let wm = {
            let wm = wi * ior;
            if cosi == 0.0 || coso == 0.0 || wm.length_squared() == 0.0 {
                return 0.0;
            };
            wm.signum() * wm.normalize()
        };

        if wi.dot(wm) * cosi < 0.0 || wo.dot(wm) * coso < 0.0 {
            return 0.0;
        }

        let r = fresnel_dielectric(wo.dot(wm), self.ior);
        let t = 1.0 - r;

        if reflect {
            distrib.pdf(wo, wm) / (4.0 * f32::abs(wo.dot(wm))) * r / (r + t)
        } else {
            let dwm_dwi = f32::abs(wi.dot(wm)) / (wi.dot(wm) + wi.dot(wm) / ior).powi(2);
            distrib.pdf(wo, wm) * dwm_dwi * t / (r + t)
        }
    }

    fn sample_f(&self, wo: Vec3, uv: Sample2D, w: Sample1D) -> Option<BxDFSample> {
        let distrib = distributions::IsotropicTrowbridgeReitzDistribution {
            alpha: self.roughness,
        };

        if self.ior == 1.0 || distrib.is_smooth() {
            // perfect specular
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
                let (wi, _) = wo.refract(Vec3::Z, self.ior)?;
                debug_assert!(!wi.is_nan());
                Some(BxDFSample {
                    wi,
                    f: (t / wi.z.abs()) * WHITE,
                    pdf: t / (r + t),
                })
            }
        } else {
            // rough
            let wm = distrib.sample_wm(wo, uv);
            trace!("wm {wm:?} wo {wo:?}");
            let r = fresnel_dielectric(wo.dot(wm), self.ior);
            let t = 1.0 - r;
            if w[0] < r / (r + t) {
                // reflection
                let wi = wo.reflect(wm);
                trace!("{:?}  {:?} {:?}", wo, wm, wi);
                if !wo.same_hemishpere(wi) {
                    return None;
                };

                let f = distrib.d(wm) * distrib.g(wo, wi) * r / (4.0 * wi.z * wo.z) * WHITE;

                let pdf = distrib.pdf(wo, wm) / (4.0 * f32::abs(wo.dot(wm))) * r / (r + t);
                debug_assert!(!pdf.is_nan());
                Some(BxDFSample { wi, f, pdf })
            } else {
                // transmission
                let (wi, ior) = wo.refract(wm, self.ior)?;
                if wi.same_hemishpere(wo) || wi.z == 0.0 {
                    return None;
                }

                let denom = (wi.dot(wm) + wo.dot(wm) / ior).powi(2);
                let dwm_dwi = f32::abs(wi.dot(wm)) / denom;

                let f = t
                    * distrib.d(wm)
                    * distrib.g(wo, wi)
                    * f32::abs(wo.dot(wm) * wi.dot(wm) / (wi.z * wo.z * denom))
                    * WHITE;

                let pdf = distrib.pdf(wo, wm) * dwm_dwi * t / (r + t);
                debug_assert!(!pdf.is_nan());
                Some(BxDFSample { wi, f, pdf })
            }
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
