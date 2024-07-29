pub mod aggregate;
pub mod camera;
pub mod color;
pub mod integrators;
pub mod loader;
pub mod material;
pub mod math;
pub mod ray;
pub mod renderer;
pub mod scene;
pub mod shape;
pub mod utils;

pub use rand_xoshiro::Xoshiro256StarStar as Rng;

use renderer::World;

pub struct Ctx<'a> {
    pub rng: Rng,
    pub world: &'a World<'a>,
}
