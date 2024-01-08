use crate::{ray::Ray, renderer::{RayResult, Renderer}};

mod basic;

pub trait Integrator: Send + Sync {
    fn throw_ray(&self, renderer: &Renderer, ray: Ray, depth: u32) -> RayResult;
}

pub use basic::BasicIntegrator;
