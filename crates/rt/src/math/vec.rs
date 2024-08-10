pub use glam::Vec2;
pub use glam::Vec3;

use crate::color::Rgb;

pub trait Vec3Ext: Sized {
    fn same_hemishpere(&self, w: Vec3) -> bool;
    fn into_non_zero(self, eps: f32) -> Option<Self>;
    fn refract(self, normal: Vec3, ior: f32) -> Option<(Vec3, f32)>;
    fn reflect(self, normal: Vec3) -> Vec3;
    fn same_direction(self, other: Self) -> Self;
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn same_hemishpere(&self, w: Vec3) -> bool {
        self.z * w.z > 0.0
    }

    fn into_non_zero(self, eps: f32) -> Option<Self> {
        use super::float::FloatAsExt;
        self.length_squared()
            .into_non_zero(eps * eps)
            .and(Some(self))
    }

    fn reflect(self, normal: Vec3) -> Vec3 {
        -self + 2.0 * self.dot(normal) * normal
    }

    // From outside  to inside => wo  and normal are in the same hemisphere
    // Otherwise from outside to inside and the ior and changed accordingly
    fn refract(self, normal: Vec3, ior: f32) -> Option<(Vec3, f32)> {
        let (normal, cosi, ior) = {
            let cosi = self.dot(normal);
            if cosi >= 0.0 {
                (normal, cosi, ior)
            } else {
                // From inside to outside
                (-normal, -cosi, 1.0 / ior)
            }
        };

        let cost2 = 1.0 - f32::max(0.0, 1. - cosi * cosi) / ior / ior;

        if cost2 <= 0. {
            None
        } else {
            let cost = f32::sqrt(cost2);
            Some((-self / ior + (cosi / ior - cost) * normal, ior))
        }
    }

    fn same_direction(self, other: Self) -> Self {
        if self.dot(other) > 0.0 {
            self
        } else {
            -self
        }
    }
}

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
