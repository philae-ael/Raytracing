pub use glam::Vec3;

use crate::{color::Rgb, utils::log_once::warn_once};

pub trait RgbAsVec3Ext {
    fn vec(&self) -> Vec3;
}

impl RgbAsVec3Ext for Rgb {
    fn vec(&self) -> Vec3 {
        Vec3::from_array(self.0)
    }
}

pub trait Vec3AsRgbExt {
    fn rgb(&self) -> Rgb;
}

impl Vec3AsRgbExt for Vec3 {
    fn rgb(&self) -> Rgb {
        Rgb::from_array(self.to_array())
    }
}

pub trait RefrReflVecExt {
    fn refract(self, normal: Vec3, ior: f32) -> Option<Vec3>;
    fn reflect(self, normal: Vec3) -> Vec3;
}

impl RefrReflVecExt for Vec3 {
    fn reflect(self, normal: Vec3) -> Vec3 {
        self - (2.0 * self.dot(normal) * normal)
    }

    fn refract(self, normal: Vec3, ior: f32) -> Option<Vec3> {
        let cosi = self.dot(normal);
        if cosi > 0.0 {
            warn_once!("Error during refraction: Normal and vector should be in opposite direction. They are in the same direction.");
        }

        let k = ior * ior * (1. - cosi * cosi);

        if k > 1. {
            None
        } else {
            Some(ior * (self - cosi * normal) - f32::sqrt(1. - k) * normal)
        }
    }
}

pub trait Vec3SameDirExt {
    fn same_direction(self, other: Self) -> Self;
}

impl Vec3SameDirExt for Vec3 {
    /// Return self if self and other are pointing in the same general direction (self.dot(other) > 0.0) else, returns self
    fn same_direction(self, other: Self) -> Self {
        if self.dot(other) > 0.0 {
            self
        } else {
            -self
        }
    }
}

pub trait Vec3AsNonZero: Sized {
    fn into_non_zero(self, eps: f32) -> Option<Self>;
}

impl Vec3AsNonZero for Vec3 {
    fn into_non_zero(self, eps: f32) -> Option<Self> {
        use super::float::FloatAsExt;
        self.length_squared().into_non_zero(eps * eps).and(Some(self))
    }
}
