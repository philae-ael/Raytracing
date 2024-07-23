use crate::color::Rgb;

/// Represent a serie of samples from a given discribution.
/// It is used to get an easy access to mean, variance and
#[derive(Default)]
pub struct SampleSeries {
    count: usize,
    sum: f32,
    sqsum: f32,
}

impl SampleSeries {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            sqsum: 0.0,
        }
    }
    pub fn add_sample(&mut self, sample: f32) {
        self.count += 1;
        self.sum += sample;
        self.sqsum += sample * sample;
    }
    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self {
            count: lhs.count + rhs.count,
            sum: lhs.sum + rhs.sum,
            sqsum: lhs.sqsum + rhs.sqsum,
        }
    }

    pub fn mean(&self) -> f32 {
        self.sum / self.count as f32
    }

    pub fn variance(&self) -> f32 {
        let mean = self.mean();
        let sqmean = self.sqsum / self.count as f32;

        sqmean - mean * mean
    }

    /// returns the estimated error supposing that the distribution follows a standard distribution,
    /// with a confidence of 95%
    /// If there is not enough samples, return None
    pub fn error_with_95_confidence(&self) -> Option<f32> {
        let n = self.count;

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

    pub fn merge(lhs: Self, rhs: Self) -> Self {
        Self {
            r: SampleSeries::merge(lhs.r, rhs.r),
            g: SampleSeries::merge(lhs.g, rhs.g),
            b: SampleSeries::merge(lhs.b, rhs.b),
        }
    }
}
