use crate::color::Rgb;

/// Represent a serie of samples from a given discribution.
/// It is used to get an easy access to mean, variance and
#[derive(Default)]
pub struct SampleSeries {
    samples: Vec<f32>,
}

impl SampleSeries {
    pub fn new() -> Self {
        Self { samples: vec![] }
    }
    pub fn add_sample(&mut self, sample: f32) {
        self.samples.push(sample)
    }

    pub fn mean(&self) -> f32 {
        self.samples.iter().fold(0.0, |acc, sample| acc + sample) / self.samples.len() as f32
    }

    pub fn variance(&self) -> f32 {
        let mean = self.mean();
        self.samples
            .iter()
            .fold(0.0, |acc, sample| acc + (sample - mean).powi(2))
    }

    /// returns the estimated error supposing that the distribution follows a standard distribution,
    /// with a confidence of 95%
    /// If there is not enough samples, return None
    pub fn error_with_95_confidence(&self) -> Option<f32> {
        let n = self.samples.len();

        if n < 15 {
            return None;
        };

        let c = 2.13; // from https://www.accessengineeringlibrary.com/content/book/9780071795579/back-matter/appendix4 for n=15, t.975 (0.025*2 = 0.05)
        Some(c * (self.variance() / n as f32).sqrt())
    }

    pub fn value(&self) -> f32 {
        self.mean()
    }

    pub fn is_precise_enough(&self, abs_err: f32) -> Option<f32> {
        self.error_with_95_confidence().and_then(|err| {
            if err < abs_err {
                Some(self.mean())
            } else {
                None
            }
        })
    }
}

#[derive(Default)]
pub struct RgbSeries {
    r: SampleSeries,
    g: SampleSeries,
    b: SampleSeries,
}

impl RgbSeries {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sample(&mut self, rgb: Rgb) {
        self.r.add_sample(rgb.0[0]);
        self.g.add_sample(rgb.0[1]);
        self.b.add_sample(rgb.0[2]);
    }

    pub fn is_precise_enough(&self, abs_err: f32) -> Option<Rgb> {
        let r = self.r.is_precise_enough(abs_err)?;
        let g = self.g.is_precise_enough(abs_err)?;
        let b = self.b.is_precise_enough(abs_err)?;

        Some(Rgb::from_array([r, g, b]))
    }
    pub fn mean(&self) -> Rgb {
        let r = self.r.mean();
        let g = self.g.mean();
        let b = self.b.mean();

        Rgb::from_array([r, g, b])
    }

    pub fn merge(&mut self, other: &Self) {
        for r in other.r.samples.iter() {
            self.r.add_sample(*r)
        }
        for g in other.g.samples.iter() {
            self.g.add_sample(*g)
        }
        for b in other.b.samples.iter() {
            self.b.add_sample(*b)
        }
    }
}
