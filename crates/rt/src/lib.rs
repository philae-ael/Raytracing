#![feature(float_next_up_down)]
#![feature(new_uninit)]
#![feature(allocator_api)]
#![feature(negative_impls)]

pub mod aggregate;
pub mod camera;
pub mod color;
pub mod filter;
pub mod integrators;
pub mod loader;
pub mod material;
pub mod math;
pub mod memory;
pub mod ray;
pub mod renderer;
pub mod sampler;
pub mod scene;
pub mod shape;
pub mod utils;

pub use rand_xoshiro::Xoshiro256StarStar as Rng;

pub struct Ctx<'a> {
    pub rng: Rng,
    pub world: &'a renderer::World<'a>,
    pub arena: memory::Arena<'a>,
    pub seed: Seed,
    pub sampler: &'a mut dyn sampler::Sampler,
}

#[derive(Debug, Copy, Clone, Hash)]
#[repr(C)]
pub struct Seed {
    pub seed: u64,
    pub x: u32,
    pub y: u32,
    pub sample_idx: u32,
}

impl Seed {
    pub fn into_rng(self, local_seed: u32) -> Rng {
        let mut hasher = std::hash::DefaultHasher::new();
        std::hash::Hash::hash(&self, &mut hasher);
        std::hash::Hash::hash(&local_seed, &mut hasher);
        <Rng as rand::SeedableRng>::seed_from_u64(std::hash::Hasher::finish(&hasher))
    }
}
