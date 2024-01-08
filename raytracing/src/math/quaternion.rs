use glam::Vec3;

pub use glam::Quat;

use super::float::FloatAsExt;

pub struct LookAt {
    pub direction: Vec3,
    pub forward: Vec3,
}

impl From<LookAt> for Quat {
    fn from(this: LookAt) -> Self {
        let direction = this.direction.normalize();
        let forward = this.forward.normalize();

        let cos = forward.dot(direction);
        let angle = cos.acos();
        match (cos.abs() - 1.0).as_non_zero(0.01) {
            Some(_non_0deg_cos) => {
                let axe = forward.cross(direction);
                Self::from_axis_angle(axe.normalize(), angle)
            }
            None => Self::from_axis_angle(Vec3::Y, angle),
        }
    }
}
