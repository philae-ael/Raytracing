use crate::math::vec::Vec2;
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use std::hash::{DefaultHasher, Hash, Hasher};

pub const ONE_MINUS_EPSILON: f32 = f32::next_down(1.0);

pub trait Sampler {
    /// Sample a point around the pixel located at `coords`
    fn sample_2d(&mut self) -> Vec2;

    // Max number of samples
    fn sample_count(&self) -> u32 {
        u32::MAX
    }
    fn with_sample(&mut self, _sample: u32) {}
}

/// Given a pixel coordinate (x, y), the sample is (x + 0.5, y + 0.5)
#[derive(Clone)]
pub struct DummyPixelSampler;
impl Sampler for DummyPixelSampler {
    fn sample_2d(&mut self) -> Vec2 {
        Vec2 { x: 0.5, y: 0.5 }
    }
}

fn seed_rng(x: u32, y: u32, sample: u32) -> crate::Rng {
    let mut hasher = DefaultHasher::new();
    (x, y, sample).hash(&mut hasher);
    crate::Rng::seed_from_u64(hasher.finish())
}

/// Given a pixel coordinate (x, y), the sample is taken uniformely in
/// $\left[x, x+1\right[ \times \left[y, x+1\right[$
#[derive(Clone)]
pub struct UniformSampler {
    x: u32,
    y: u32,
    rng: crate::Rng,
    uniform: Uniform<f32>,
}

impl UniformSampler {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
            rng: seed_rng(x, y, 0),
            uniform: Uniform::new(0., 1.),
        }
    }
}

impl Sampler for UniformSampler {
    fn sample_2d(&mut self) -> Vec2 {
        Vec2 {
            x: self.uniform.sample(&mut self.rng),
            y: self.uniform.sample(&mut self.rng),
        }
    }
    fn with_sample(&mut self, sample: u32) {
        self.rng = seed_rng(self.x, self.y, sample);
    }
}

#[derive(Clone)]
pub struct StratifiedSampler {
    rng: crate::Rng,
    uniform: Uniform<f32>,
    samples_x: u32,
    samples_y: u32,
    sample: u32,
    x: u32,
    y: u32,
}

impl StratifiedSampler {
    pub fn new(x: u32, y: u32, samples_x: u32, samples_y: u32) -> Self {
        Self {
            x,
            y,
            samples_x,
            samples_y,
            sample: 0,
            rng: seed_rng(x, y, 0),
            uniform: Uniform::new(0., 1.),
        }
    }
}

impl Sampler for StratifiedSampler {
    fn sample_2d(&mut self) -> Vec2 {
        // Note index is taken as sample but is should be randomly permuted
        // See PBRT p734
        let index = self.sample % self.sample_count();
        let x = (index % self.samples_x) as f32;
        let y = (index / self.samples_x) as f32;
        Vec2 {
            x: (x + self.uniform.sample(&mut self.rng)) / self.samples_x as f32,
            y: (y + self.uniform.sample(&mut self.rng)) / self.samples_y as f32,
        }
    }

    fn sample_count(&self) -> u32 {
        self.samples_x * self.samples_y
    }

    fn with_sample(&mut self, sample: u32) {
        self.sample = sample;
        self.rng = seed_rng(self.x, self.y, sample);
    }
}
