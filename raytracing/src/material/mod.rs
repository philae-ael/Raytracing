pub mod dielectric;
mod diffuse;
mod emit;
mod gooch;
pub mod material;
mod metal;
mod mix;
pub mod phong;
pub mod texture;

pub use dielectric::Dielectric;
pub use diffuse::Diffuse;
pub use emit::Emit;
pub use gooch::Gooch;
pub use material::{Material, MaterialDescriptor, MaterialId, Scattered};
pub use mix::MixMaterial;
