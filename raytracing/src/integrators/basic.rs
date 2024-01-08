use glam::Vec3;

use crate::{
    color,
    math::{distributions::sphere_uv_from_direction, vec::{RgbAsVec3Ext, Vec3AsRgbExt}},
    ray::Ray,
    renderer::{RayResult, Renderer},
    shape::{local_info, IntersectionResult, Shape},
};

use super::Integrator;

pub struct BasicIntegrator;

impl Integrator for BasicIntegrator {
    fn throw_ray(&self, renderer: &Renderer, ray: Ray, depth: u32) -> RayResult {
        let mut rng = rand::thread_rng();
        if depth == 0 {
            return RayResult::default();
        }

        // Prevent auto intersection
        let ray = Ray::new_with_range(ray.origin, ray.direction, 0.01..ray.bounds.1);

        if let IntersectionResult::Instersection(record) = renderer.objects.intersection_full(ray) {
            // On material hit
            let material = &renderer.materials[record.local_info.material.0].material;
            let scattered = material.scatter(ray, &record.local_info, &mut rng);

            let (color, ray_depth) = if let Some(ray_out) = scattered.ray_out {
                let ray_result = self.throw_ray(renderer, ray_out, depth - 1);
                (ray_result.color, ray_result.ray_depth)
            } else {
                (color::WHITE, 0.0)
            };

            let color = (color.vec() * scattered.albedo.vec()).rgb();

            RayResult {
                normal: record.local_info.normal,
                color,
                z: record.t,
                albedo: scattered.albedo,
                ray_depth: ray_depth + 1.0,
                samples_accumulated: 1,
            }
        } else {
            // Sky
            let material = &renderer.materials[renderer.options.world_material.0].material;
            let record = local_info::Full {
                pos: ray.origin,
                normal: -ray.direction,
                material: renderer.options.world_material,
                uv: sphere_uv_from_direction(-ray.direction),
            };
            let scattered = material.scatter(ray, &record, &mut rng);
            RayResult {
                normal: Vec3::ZERO,
                albedo: color::BLACK,
                color: scattered.albedo,
                z: 0.0,
                ray_depth: 0.0,
                samples_accumulated: 1,
            }
        }
    }
}
