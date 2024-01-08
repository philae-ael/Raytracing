pub mod examples;

use crate::{
    aggregate::shapelist::ShapeList,
    material::{MaterialDescriptor, MaterialId},
    shape::Shape,
};

#[derive(Default)]
pub struct Scene {
    pub objects: ShapeList,
    pub materials: Vec<MaterialDescriptor>,
}

impl Scene {
    /// Insert an object in the scene
    pub fn insert_object<T: Shape + Sync + Send + 'static>(&mut self, object: T) {
        self.objects.0.push(Box::new(object))
    }

    /// Insert a material and returns the Material ID associated with this material
    pub fn insert_material(&mut self, material: MaterialDescriptor) -> MaterialId {
        self.materials.push(material);
        MaterialId(self.materials.len() - 1)
    }
}