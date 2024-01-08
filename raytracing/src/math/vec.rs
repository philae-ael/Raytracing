pub use glam::Vec3;
use image::Rgb;

pub trait RgbAsVec3Ext {
    fn vec(&self) -> Vec3;
}

impl RgbAsVec3Ext for Rgb<f32> {
    fn vec(&self) -> Vec3 {
        Vec3::from_array(self.0)
    }
}

pub trait Vec3AsRgbExt {
    fn rgb(&self) -> Rgb<f32>;
}

impl Vec3AsRgbExt for Vec3 {
    fn rgb(&self) -> Rgb<f32> {
        Rgb(self.to_array())
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

    fn refract(self, mut normal: Vec3, ior: f32) -> Option<Vec3> {
        let mut cosi = self.dot(normal);
        let mut etai = 1.;
        let mut etat = ior;
        if cosi < 0.0 {
            cosi = -cosi;
        } else {
            (etat, etai) = (etai, etat);
            normal = -normal;
        }
        let eta = etai / etat;
        let k = 1. - eta * eta * (1. - cosi * cosi);

        if k < 0. {
            None
        } else {
            Some(eta * self + (eta * cosi - f32::sqrt(k)) * normal)
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
    fn as_non_zero(self, eps: f32) -> Option<Self>;
}

impl Vec3AsNonZero for Vec3 {
    fn as_non_zero(self, eps: f32) -> Option<Self> {
        use super::float::FloatAsExt;
        self.length_squared().as_non_zero(eps * eps).and(Some(self))
    }
}
