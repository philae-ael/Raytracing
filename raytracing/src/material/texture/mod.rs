use crate::color::Rgb;


pub type Uv = [f32; 2];
pub trait Texture: Sync + Send {
    fn color(&self, uv: Uv) -> Rgb;
}

pub struct Uniform(pub Rgb);

impl Texture for Uniform {
    fn color(&self, _: Uv) -> Rgb {
        self.0
    }
}

pub struct Checker {
    pub odd: Box<dyn Texture>,
    pub even: Box<dyn Texture>,
}

impl Texture for Checker {
    fn color(&self, uv: Uv) -> Rgb {
        let fu = 10.;
        let fv = 10.;
        let wu = std::f32::consts::TAU * fu;
        let wv = std::f32::consts::TAU * fv;
        let even = f32::cos(wu * uv[0]) * f32::cos(wv * uv[1]) > 0.0;
        let uv = [uv[0] / fu, uv[1] / fv];
        if even {
            self.even.color(uv)
        } else {
            self.odd.color(uv)
        }
    }
}
