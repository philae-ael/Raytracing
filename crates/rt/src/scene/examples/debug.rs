use glam::Vec3;

use crate::{color::Rgb, material::Gooch, math::point::Point, scene::SceneT};

pub struct DebugScene;

impl DebugScene {
    pub fn insert_into<S: SceneT>(scene: &mut S) {
        let default_material = scene.insert_material(crate::material::MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(Gooch {
                diffuse: Rgb::from_array([1.0, 0., 0.]),
                smoothness: 20.0,
                light_dir: Vec3::new(-1.0, -1.0, 0.0),
                yellow: Rgb::from_array([0.8, 0.8, 0.0]),
                blue: Rgb::from_array([0.0, 0.0, 0.8]),
            }),
        });

        scene.insert_sphere(default_material, Point::new(0.0, 0.0, -1.0), 0.3);
        scene.insert_sphere(default_material, Point::new(0.5, 0.0, -1.0), 0.3);
    }
}
