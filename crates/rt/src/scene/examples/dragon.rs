use std::f32::consts::PI;

use glam::{Quat, Vec3};

use crate::{
    color::Rgb,
    loader::ObjLoaderExt,
    material::{DielectricBxDF, DiffuseBxDF},
    math::{point::Point, transform::Transform},
    scene::SceneT,
};

pub struct DragonScene;

impl DragonScene {
    pub fn insert_into<S: SceneT>(scene: &mut S) {
        let glass = scene.insert_material(crate::material::MaterialDescriptor {
            label: None,
            material: Box::new(DielectricBxDF {
                ior: 1.5,
                roughness: 0.2,
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

        scene.insert_light(crate::material::LightDescriptor {
            label: None,
            light_pos: Point::new(10.2, 80.0, 75.0),
        });

        let ball = scene.insert_material(crate::material::MaterialDescriptor {
            label: None,
            material: Box::new(DiffuseBxDF {
                albedo: Rgb::from_array([0.2, 0.1, 0.5]),
            }),
        });
        scene.insert_sphere(ball, Point::new(-0.7, -0.2, -1.9), 0.8);
    }
}
