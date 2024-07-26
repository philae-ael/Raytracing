pub mod examples;

use glam::Vec3;

use crate::{
    aggregate::shapelist::{ShapeList, ShapeListEntry},
    material::{Material, MaterialDescriptor, MaterialId},
    math::point::Point,
    shape::Shape,
};

pub struct Scene {
    pub objects: ShapeList,
    pub materials: Vec<MaterialDescriptor>,
    pub sky_material: MaterialId,
    pub lights: Vec<Point>,
}

impl Scene {
    pub fn new<M: Material + 'static + Send + Sync>(material: M) -> Self {
        Self {
            materials: vec![MaterialDescriptor {
                label: Some("Sky".to_owned()),
                material: Box::new(material),
            }],
            sky_material: MaterialId(0),
            objects: Default::default(),
            lights: Default::default(),
        }
    }
    /// Insert an object in the scene
    pub fn insert_object<T: Shape + Sync + Send + 'static>(&mut self, object: T) {
        self.objects.0.push(ShapeListEntry::Shape(Box::new(object)))
    }

    pub fn insert_shape_list(&mut self, list: ShapeList) {
        self.objects.0.push(ShapeListEntry::List(list))
    }

    /// Insert a light in the scene
    pub fn insert_light(&mut self, light_pos: Point) {
        self.lights.push(light_pos);
    }

    /// Insert a material and returns the Material ID associated with this material
    pub fn insert_material<M: Material + Sync + Send + 'static>(
        &mut self,
        label: Option<String>,
        material: M,
    ) -> MaterialId {
        self.materials.push(MaterialDescriptor {
            label,
            material: Box::new(material),
        });
        MaterialId(self.materials.len() - 1)
    }
}
