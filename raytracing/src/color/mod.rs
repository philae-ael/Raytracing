use std::marker::PhantomData;

pub mod colorspace;

#[derive(Clone, Copy, Debug)]
pub struct Color<S: colorspace::Colorspace>(pub [f32; 3], PhantomData<S>);

impl<S: colorspace::Colorspace> Color<S> {
    pub const fn from_array(arr: [f32; 3]) -> Self {
        Self(arr, PhantomData)
    }

    pub const fn to_array(self) -> [f32; 3] {
        self.0
    }

    pub fn to_byte_array(self) -> [u8; 3] {
        self.0.map(|c| (c * 255. + 0.5) as u8)
    }

    pub fn convert<S2: colorspace::Colorspace>(self) -> Color<S2> {
        Color::from_array(S2::from_cie_xyz(S::to_cie_xyz(self.to_array())))
    }
}

impl<S: colorspace::Colorspace> From<[f32; 3]> for Color<S> {
    fn from(val: [f32; 3]) -> Self {
        Color::<S>::from_array(val)
    }
}

#[allow(non_camel_case_types)]
pub type sRgb = Color<colorspace::sRGB>;
impl sRgb {
    pub fn to_rgb(self) -> Rgb {
        self.to_array()
            .map(colorspace::Linear_sRGB::from_srgb)
            .into()
    }
}
pub type Rgb = Color<colorspace::Linear_sRGB>;
impl Rgb {
    pub fn to_srgb(self) -> sRgb {
        self.to_array()
            .map(colorspace::sRGB::from_linear_rgb)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Luma(pub f32);

impl<S: colorspace::Colorspace> From<Color<S>> for Luma {
    fn from(val: Color<S>) -> Self {
        Luma(val.convert::<colorspace::CIE_XYZ>().0[1])
    }
}

impl From<sRgb> for image::Rgb<f32> {
    fn from(val: sRgb) -> Self {
        image::Rgb(val.to_array())
    }
}

impl From<Luma> for image::Luma<f32> {
    fn from(val: Luma) -> Self {
        image::Luma([val.0])
    }
}

pub mod linear {
    use super::Rgb;

    pub const WHITE: Rgb = Rgb::from_array([1.0, 1.0, 1.0]);
    pub const BLACK: Rgb = Rgb::from_array([0.0, 0.0, 0.0]);
    pub const RED: Rgb = Rgb::from_array([1.0, 0.0, 0.0]);
    pub const GREEN: Rgb = Rgb::from_array([0.0, 1.0, 0.0]);
    pub const BLUE: Rgb = Rgb::from_array([0.0, 0.0, 1.0]);
}

pub enum MixMode {
    Add,
    Mul,
}
