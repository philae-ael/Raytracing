use glam::Vec3;

use crate::{
    color,
    material::{dielectric::Dielectric, texture, Diffuse, MaterialDescriptor},
    scene::Scene,
    shape::Sphere,
};

pub struct SpheresScene;
impl From<SpheresScene> for Scene {
    fn from(_: SpheresScene) -> Self {
        let mut scene = Scene::default();
        let diffuse = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(color::BLUE)),
            }),
        });
        let glass = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Dielectric {
                texture: Box::new(texture::Uniform(color::WHITE)),
                ior: 0.9,
                invert_normal: false,
            }),
        });

        scene.insert_object(Sphere {
            center: Vec3::new(0.0, 0.4, -1.0),
            radius: 0.5,
            material: diffuse,
        });
        scene.insert_object(Sphere {
            center: Vec3::new(0.0, 0.0, -0.7),
            radius: 0.3,
            material: glass,
        });

        scene.insert_light(Vec3::new(0.0, 0.2, -0.1));
        scene
    }
}
