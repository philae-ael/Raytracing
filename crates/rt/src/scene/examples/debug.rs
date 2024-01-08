use glam::Vec3;

use crate::{color::Rgb, material::{Gooch, texture, Emit}, math::point::Point, scene::Scene, shape::Sphere};

pub struct DebugScene;

impl From<DebugScene> for Scene {
    fn from(_: DebugScene) -> Self {
        let mut scene = Scene::new(Emit {
            texture: Box::new(texture::Uniform(Rgb::from_array([0.3, 0.3, 0.3]))),
        });
        let default_material = scene.insert_material(
            Some("Goosh - Default".to_string()),
            Gooch {
                diffuse: Rgb::from_array([1.0, 0., 0.]),
                smoothness: 20.0,
                light_dir: Vec3::new(-1.0, -1.0, 0.0),
                yellow: Rgb::from_array([0.8, 0.8, 0.0]),
                blue: Rgb::from_array([0.0, 0.0, 0.8]),
            },
        );

        scene.insert_object(Sphere {
            center: Point::new(0.0, 0.0, -1.0),
            radius: 0.3,
            material: default_material,
        });
        scene.insert_object(Sphere {
            center: Point::new(0.5, 0.0, -1.0),
            radius: 0.3,
            material: default_material,
        });
        scene
    }
}
