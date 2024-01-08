use crate::{
    color,
    math::vec::{RgbAsVec3Ext, Vec3AsRgbExt},
    ray::Ray,
    renderer::{RayResult, Renderer},
    shape::{IntersectionResult, Shape},
};

use super::Integrator;

pub struct BasicIntegrator {
    pub max_depth: u32,
}

impl Integrator for BasicIntegrator {
    fn ray_cast(&self, renderer: &Renderer, ray: Ray, depth: u32) -> RayResult {
        let mut rng = rand::thread_rng();
        if depth == self.max_depth {
            return RayResult::default();
        }

        // Prevent auto intersection
        let ray = Ray::new_with_range(ray.origin, ray.direction, 0.01..ray.bounds.1);

        let IntersectionResult::Instersection(record) = renderer.objects.intersection_full(ray) else  {
            return self.sky_ray(renderer, ray);
        };

        // On material hit
        let material = &renderer.materials[record.local_info.material.0].material;
        let scattered = material.scatter(ray, &record.local_info, &mut rng);

        let color = if let Some(ray_out) = scattered.ray_out {
            let ray_result = self.ray_cast(renderer, ray_out, depth + 1);
            ray_result.color
        } else {
            color::WHITE
        };

        let color = (color.vec() * scattered.albedo.vec()).rgb();

        RayResult {
            normal: record.local_info.normal,
            albedo: scattered.albedo,
            color,
            z: record.t,
            ray_depth: (depth + 1) as f32,
            samples_accumulated: 1,
        }
    }
}
