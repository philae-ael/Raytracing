use glam::Vec3;
use log::trace;
use rand::prelude::Distribution;

use crate::{
    color::linear::BLACK,
    material::{BxDFSample, BSDF},
    math::{distributions::Samples, vec::RgbAsVec3Ext},
    ray::Ray,
    renderer::RayResult,
    shape::IntersectionResult,
    Ctx,
};

use super::Integrator;

pub struct PathTracer {
    pub max_depth: u32,
}

impl Integrator for PathTracer {
    fn ray_cast(&self, ctx: &mut Ctx, ray: Ray, depth: u32) -> RayResult {
        let uniform = rand::distributions::Uniform::new(0.0, 1.0);
        if depth == self.max_depth {
            return RayResult::default();
        }
        trace!("depth {depth:?}");

        let ray = Ray::new_with_range(ray.origin, ray.direction, 0.00001..ray.bounds.1);

        let isect = ctx.world.objects.intersection_full(ray);
        let IntersectionResult::Intersection(record) = isect else {
            return self.sky_ray(ctx, ray);
        };

        let material = &ctx.world.materials[record.local_info.material.0].material;
        // TODO: The material should do it
        let bsdf = BSDF::new(record.local_info.normal, material.as_ref());

        let wo = -ray.direction;
        let sampled = bsdf
            .sample_f(
                wo,
                Samples([uniform.sample(&mut ctx.rng), uniform.sample(&mut ctx.rng)]),
                Samples([uniform.sample(&mut ctx.rng)]),
            )
            .unwrap_or(BxDFSample {
                wi: Vec3::ZERO,
                f: BLACK,
                pdf: 1.0,
            });
        trace!("sampled {:?}", sampled);

        let fcos = record.local_info.normal.dot(sampled.wi).abs() * sampled.f;
        trace!("fcos {fcos:?}");
        let (li, ray_depth) = if fcos.vec().max_element().abs() != 0.0 {
            let ray_result =
                self.ray_cast(ctx, Ray::new(record.local_info.pos, sampled.wi), depth + 1);
            (
                material.le() + 1.0 / sampled.pdf * fcos * ray_result.color,
                ray_result.ray_depth,
            )
        } else {
            (material.le(), 0.0)
        };

        trace!("li {:?}", li);
        trace!("le {:?}", material.le());

        RayResult {
            normal: record.local_info.normal,
            position: record.local_info.pos,
            albedo: sampled.f,
            color: li,
            z: record.t,
            ray_depth: ray_depth + record.t,
            samples_accumulated: 1,
        }
    }
}
