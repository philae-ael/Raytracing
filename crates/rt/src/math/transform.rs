use glam::{Quat, Vec3};

use super::point::Point;

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

pub trait Transformer<T> {
    fn apply(&self, v: T) -> T;
}

impl Transform {
    const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        scale: Vec3::ONE,
        rot: Quat::IDENTITY,
    };
}

impl Transform {
    pub fn into_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rot, self.translation)
    }
}

impl Transformer<Vec3> for Transform {
    /// Apply rotation then scale but not translation !
    fn apply(&self, v: Vec3) -> Vec3 {
        let rotated = self.rot.mul_vec3(v);

        self.scale * rotated
    }
}

impl Transformer<Point> for Transform {
    /// Apply rotation then scale then translation
    fn apply(&self, v: Point) -> Point {
        let rotated = self.rot.mul_vec3(v.vec());
        let rotated_scaled = self.scale * rotated;
        Point(rotated_scaled) + self.translation
    }
}

/// Represent an orthonormal frame
pub struct Frame {
    frame: glam::Mat3,
}

impl Frame {
    /// Construct a Frame from a single vector using the algorithm described in
    /// “Building an Orthonormal Basis, Revisited (JCGT).” Accessed August 6, 2024. https://jcgt.org/published/0006/01/01/.
    /// n is expected to be normalized and will be used as the +z axis
    pub fn new(n: Vec3) -> Self {
        let sign = f32::signum(n.z);
        let a = -1.0 / (sign + n.z);
        let b = n.x * n.y * a;

        let this = Self {
            frame: glam::Mat3::from_cols(
                Vec3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
                Vec3::new(b, sign + n.y * n.y * a, -n.y),
                n,
            ),
        };
        debug_assert!(
            (this.frame * this.frame.transpose() - glam::Mat3::IDENTITY)
                .to_cols_array()
                .into_iter()
                .reduce(f32::max)
                .unwrap_or(0.0)
                .abs()
                < 1e5
        );

        this
    }

    pub fn to_local(&self, global: Vec3) -> Vec3 {
        self.frame.transpose() * global
    }

    pub fn from_local(&self, local: Vec3) -> Vec3 {
        self.frame * local
    }

    pub fn x(&self) -> Vec3 {
        self.frame.col(0)
    }
    pub fn y(&self) -> Vec3 {
        self.frame.col(0)
    }
    pub fn z(&self) -> Vec3 {
        self.frame.col(0)
    }
}
