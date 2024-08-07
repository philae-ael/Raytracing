use crate::{ray::Ray, renderer::RayResult, Ctx};

mod basic;

pub trait Integrator: Send + Sync {
    fn ray_cast(&self, ctx: &mut Ctx, ray: Ray, depth: u32) -> RayResult;
    fn sky_ray(&self, _ctx: &mut Ctx, _ray: Ray) -> RayResult {
        // let material = &ctx.world.materials[ctx.world.world_material.0].material;
        // let record = local_info::Full {
        //     pos: ray.origin,
        //     normal: -ray.direction,
        //     material: ctx.world.world_material,
        //     uv: sphere_uv_from_direction(-ray.direction),
        // };

        // let scattered = material.scatter(ray, &record, &mut ctx.rng);
        RayResult {
            color: [10.0, 10.0, 10.0].into(),
            samples_accumulated: 1,
            ..Default::default()
        }
    }
}

pub use basic::RandomWalkIntegrator;
