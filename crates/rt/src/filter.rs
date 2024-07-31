use crate::math::{float::FloatAsExt, vec::Vec2};

pub struct FilterSample {
    pub coords: Vec2,
    pub weight: f32,
}
pub trait Filter {
    fn sample(&self, sample: Vec2) -> FilterSample;
}

pub struct DummyFilter;
impl Filter for DummyFilter {
    fn sample(&self, sample: Vec2) -> FilterSample {
        FilterSample {
            coords: sample,
            weight: 1.0,
        }
    }
}
pub struct BoxFilter {
    pub radius: Vec2,
}
impl Filter for BoxFilter {
    fn sample(&self, sample: Vec2) -> FilterSample {
        FilterSample {
            coords: Vec2 {
                x: sample.x.lerp(-self.radius.x, self.radius.x),
                y: sample.y.lerp(-self.radius.y, self.radius.y),
            },
            weight: 1.0,
        }
    }
}

pub struct TriangleFilter {
    pub radius: Vec2,
}
impl Filter for TriangleFilter {
    fn sample(&self, coords: Vec2) -> FilterSample {
        // TODO: this seems wrong
        fn sample_tent(c: f32) -> f32 {
            if c <= 0.5 {
                f32::sqrt(2.0 * c)
            } else {
                1.0 - f32::sqrt(2.0 - 2.0 * c)
            }
        }
        FilterSample {
            coords: Vec2 {
                x: self.radius.x * (2.0 * sample_tent(coords.x) - 1.0),
                y: self.radius.y * (2.0 * sample_tent(coords.y) - 1.0),
            },
            weight: 1.0,
        }
    }
}
