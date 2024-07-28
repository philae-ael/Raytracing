pub mod examples;

use crate::{
    material::{LightDescriptor, MaterialDescriptor, MaterialId},
    math::point::Point,
};

pub trait SceneT {
    type GeometryHandle;

    fn insert_material(&mut self, mat: MaterialDescriptor) -> MaterialId;
    fn insert_light(&mut self, light: LightDescriptor);
    fn insert_mesh(
        &mut self,
        material: MaterialId,
        vertices: &[[f32; 3]],
        indices: &[[u32; 3]],
    ) -> Self::GeometryHandle;

    fn insert_sphere(
        &mut self,
        material: MaterialId,
        origin: Point,
        radius: f32,
    ) -> Self::GeometryHandle;
}
