use crate::{
    math::distributions::sphere_uv_from_direction, ray::Ray, renderer::RayResult,
    shape::local_info, Ctx,
};

mod basic;
mod whitted;

pub trait Integrator: Send + Sync {
    fn ray_cast(&self, ctx: &mut Ctx, ray: Ray, depth: u32) -> RayResult;
    fn sky_ray(&self, ctx: &mut Ctx, ray: Ray) -> RayResult {
        let material = &ctx.world.materials[ctx.world.world_material.0].material;
        let record = local_info::Full {
            pos: ray.origin,
            normal: -ray.direction,
            material: ctx.world.world_material,
            uv: sphere_uv_from_direction(-ray.direction),
        };

        let scattered = material.scatter(ray, &record, &mut ctx.rng);
        RayResult {
            color: scattered.albedo,
            samples_accumulated: 1,
            ..Default::default()
        }
    }
}

pub use basic::BasicIntegrator;
pub use whitted::WhittedIntegrator;
