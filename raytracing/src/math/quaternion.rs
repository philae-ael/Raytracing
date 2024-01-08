use glam::Vec3;

pub use glam::Quat;

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
        if f32::abs(cos.abs() - 1.0) <= 0.01 {
            return Self::from_axis_angle(Vec3::Y, angle);
        }
        let axe = forward.cross(direction);
        Self::from_axis_angle(axe.normalize(), angle)
    }
}
