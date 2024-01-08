use image::Rgb;

pub type Uv = [f64; 2];
pub trait Texture : Sync + Send{
    fn color(&self, uv: Uv) -> Rgb<f64>;
}

pub struct Uniform (pub Rgb<f64>);

impl Texture for Uniform {
    fn color(&self, _: Uv) -> Rgb<f64> {
        self.0
    }
}

pub struct Checker {
    pub odd: Box<dyn Texture>, 
    pub even: Box<dyn Texture>, 
}

impl Texture for Checker {
    fn color(&self, uv: Uv) -> Rgb<f64> {
        let fu = 10.;
        let fv = 10.;
        let wu = std::f64::consts::TAU * fu;
        let wv = std::f64::consts::TAU * fv;
        let even = f64::cos(wu*uv[0]) * f64::cos(wv*uv[1]) > 0.0; 
        let uv  = [uv[0]/fu, uv[1]/fv];
        if even {
            self.even.color(uv)
        }else {
            self.odd.color(uv)
        }
        
    }
}


