use glam::Vec3;

use crate::{
    color::Rgb,
    material::{dielectric::Dielectric, texture, Diffuse, Emit, MaterialDescriptor, MixMaterial},
    math::vec::Vec3AsRgbExt,
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
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.9, 0.7]))),
            }),
        });
        let diffuse2 = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.3, 0.7]))),
            }),
        });
        let glass = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Dielectric {
                texture: Box::new(texture::Uniform(Rgb::from_array([1.0, 1.0, 1.0]))),
                ior: 1.5,
            }),
        });
        let light = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(MixMaterial {
                p: 0.2,
                mat1: Emit {
                    texture: Box::new(texture::Uniform(Vec3::splat(15.0).rgb())),
                },
                mat2: Diffuse {
                    texture: Box::new(texture::Uniform(Rgb::from_array([0.4, 0.5, 0.3]))),
                },
            }),
        });

        scene.insert_object(Sphere {
            center: Vec3::new(0.0, 0.2, -1.5),
            radius: 0.3,
            material: diffuse,
        });
        scene.insert_object(Sphere {
            center: Vec3::new(0.0, -0.1, -0.3),
            radius: 0.1,
            material: glass,
        });
        scene.insert_object(Sphere {
            center: Vec3::new(0.0, -1000.2, 0.0),
            radius: 1000.0,
            material: diffuse2,
        });

        scene.insert_light(Vec3::new(0.0, 0.2, -0.1));
        scene.insert_object(Sphere {
            center: Vec3::new(3.3, 3.3, 1.2),
            radius: 3.0,
            material: light,
        });
        scene
    }
}
