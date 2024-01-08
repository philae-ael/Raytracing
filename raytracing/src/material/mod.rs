mod dielectric;
mod diffuse;
mod emit;
mod gooch;
mod material;
mod metal;
mod mix;
mod phong;
pub mod texture;

pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use emit::Emit;
pub use gooch::Gooch;
pub use material::{Material, MaterialDescriptor, MaterialId, Scattered};
pub use metal::Metal;
pub use mix::MixMaterial;
