use std::f32::consts::FRAC_1_PI;

use rand::prelude::Distribution;

use crate::{
    material::{Scattered, BSDF},
    math::{
        distributions::{Samplable, Samples, UniformUnitSphere3},
        vec::RgbAsVec3Ext,
    },
    ray::Ray,
    renderer::RayResult,
    shape::IntersectionResult,
    Ctx,
};

use super::Integrator;

pub struct RandomWalkIntegrator {
    pub max_depth: u32,
}

impl Integrator for RandomWalkIntegrator {
    fn ray_cast(&self, ctx: &mut Ctx, ray: Ray, depth: u32) -> RayResult {
        if depth == self.max_depth {
            return RayResult::default();
        }

        let ray = Ray::new_with_range(ray.origin, ray.direction, 0.00001..ray.bounds.1);

        let isect = ctx.world.objects.intersection_full(ray);
        let IntersectionResult::Intersection(record) = isect else {
            return self.sky_ray(ctx, ray);
        };

        let material = &ctx.world.materials[record.local_info.material.0].material;
        // TODO: The material should do it
        let bsdf = BSDF::new(record.local_info.normal, material.as_ref());

        let uniform = rand::distributions::Uniform::new(0.0, 1.0);
        let wo = -ray.direction;
        let wi = UniformUnitSphere3.sample_with(Samples([
            uniform.sample(&mut ctx.rng),
            uniform.sample(&mut ctx.rng),
        ]));

        let s = bsdf.f(wo, wi);

        let scattered = Scattered {
            albedo: s,
            ray_out: Some(Ray::new(record.local_info.pos, wi)),
        };

        let ray_result = self.ray_cast(ctx, Ray::new(record.local_info.pos, wi), depth + 1);

        let fcos = record.local_info.normal.dot(wi).abs() * scattered.albedo;
        let li = if fcos.vec().max_element().abs() != 0.0 {
            material.le() + FRAC_1_PI / 4.0 * fcos * ray_result.color
        } else {
            material.le()
        };

        RayResult {
            normal: record.local_info.normal,
            position: record.local_info.pos,
            albedo: scattered.albedo,
            color: li,
            z: record.t,
            ray_depth: ray_result.ray_depth + record.t,
            samples_accumulated: 1,
        }
    }
}
