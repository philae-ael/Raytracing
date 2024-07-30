#![feature(new_uninit)]
#![feature(allocator_api)]
#![feature(negative_impls)]

pub mod aggregate;
pub mod camera;
pub mod color;
pub mod integrators;
pub mod loader;
pub mod material;
pub mod math;
pub mod memory;
pub mod ray;
pub mod renderer;
pub mod scene;
pub mod shape;
pub mod utils;

pub use rand_xoshiro::Xoshiro256StarStar as Rng;

pub struct Ctx<'a> {
    pub rng: Rng,
    pub world: &'a renderer::World<'a>,
    pub arena: memory::Arena<'a>,
}
