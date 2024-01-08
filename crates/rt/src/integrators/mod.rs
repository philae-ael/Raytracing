use crate::{
    math::distributions::sphere_uv_from_direction,
    ray::Ray,
    renderer::{RayResult, World},
    shape::local_info,
};

mod basic;
mod whitted;

pub trait Integrator: Send + Sync {
    fn ray_cast(&self, world: &World, ray: Ray, depth: u32) -> RayResult;
    fn sky_ray(&self, world: &World, ray: Ray) -> RayResult {
        let mut rng = rand::thread_rng();

        let material = &world.materials[world.world_material.0].material;
        let record = local_info::Full {
            pos: ray.origin,
            normal: -ray.direction,
            material: world.world_material,
            uv: sphere_uv_from_direction(-ray.direction),
        };

        let scattered = material.scatter(ray, &record, &mut rng);
        RayResult {
            color: scattered.albedo,
            samples_accumulated: 1,
            ..Default::default()
        }
    }
}

pub use basic::BasicIntegrator;
pub use whitted::WhittedIntegrator;
