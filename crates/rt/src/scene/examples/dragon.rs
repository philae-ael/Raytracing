use std::f32::consts::PI;

use glam::{Quat, Vec3};

use crate::{
    color::Rgb,
    loader::ObjLoaderExt,
    material::{texture, Dielectric, Diffuse, Emit},
    math::{point::Point, transform::Transform},
    scene::{Scene, SceneT},
    shape::Sphere,
};

pub struct DragonScene;

impl From<DragonScene> for Scene {
    fn from(_: DragonScene) -> Self {
        let mut scene = Scene::new(Emit {
            texture: Box::new(texture::Uniform(Rgb::from_array([0.01, 0.01, 0.01]))),
        });

        let c = Box::new(texture::Uniform(Rgb::from_array([0.5, 0.8, 0.9])));
        let glass = scene.insert_material(
            None,
            Dielectric {
                texture: c,
                ior: 1.5,
            },
        );

        scene.load_obj(
            "obj/dragon.obj",
            Transform {
                translation: Vec3::new(0.0, 0.0, -1.0),
                scale: 0.01 * Vec3::ONE,
                rot: Quat::from_axis_angle(Vec3::Y, 1.1 * PI),
            },
            glass,
        );

        let light = scene.insert_material(
            None,
            Emit {
                texture: Box::new(texture::Uniform(Rgb::from_array([4.0, 4.0, 4.0]))),
            },
        );
        scene.insert_object(Sphere {
            center: Point::new(10.2, 80.0, 75.0),
            radius: 100.0,
            material: light,
        });

        let ball = scene.insert_material(
            None,
            Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.1, 0.5]))),
            },
        );
        scene.insert_object(Sphere {
            center: Point::new(-0.7, -0.2, -1.9),
            radius: 0.8,
            material: ball,
        });

        scene
    }
}

impl DragonScene {
    pub fn insert_into<S: SceneT>(scene: &mut S) {
        let c = Box::new(texture::Uniform(Rgb::from_array([0.5, 0.8, 0.9])));
        let glass = scene.insert_material(crate::material::MaterialDescriptor {
            label: None,
            material: Box::new(Dielectric {
                texture: c,
                ior: 1.5,
            }),
        });

        scene.load_obj(
            "obj/dragon.obj",
            Transform {
                translation: Vec3::new(0.0, 0.0, -1.0),
                scale: 0.01 * Vec3::ONE,
                rot: Quat::from_axis_angle(Vec3::Y, 1.1 * PI),
            },
            glass,
        );

        let light = scene.insert_material(crate::material::MaterialDescriptor {
            label: None,
            material: Box::new(Emit {
                texture: Box::new(texture::Uniform(Rgb::from_array([4.0, 4.0, 4.0]))),
            }),
        });
        scene.insert_sphere(light, Point::new(10.2, 80.0, 75.0), 100.0);

        let ball = scene.insert_material(crate::material::MaterialDescriptor {
            label: None,
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.1, 0.5]))),
            }),
        });
        scene.insert_sphere(ball, Point::new(-0.7, -0.2, -1.9), 0.8);
    }
}
