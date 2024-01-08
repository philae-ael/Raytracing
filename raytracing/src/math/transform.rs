use glam::{Quat, Vec3};

/// Represents a transformation as translation + scale + rot
pub struct Transform {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rot: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// Apply rotation then scale then translation
    pub fn apply(&self, v: Vec3) -> Vec3 {
        let rotated = self.rot.mul_vec3(v);
        let rotated_scaled = self.scale * rotated;
        rotated_scaled + self.translation
    }

    const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        scale: Vec3::ONE,
        rot: Quat::IDENTITY,
    };
}
