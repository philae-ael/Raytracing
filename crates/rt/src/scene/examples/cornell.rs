use glam::{Quat, Vec3};

use crate::scene::SceneT;
use crate::{
    color::Rgb,
    loader::ObjLoaderExt,
    material::{Gooch, LightDescriptor, MaterialDescriptor},
    math::{point::Point, transform::Transform},
};

pub struct CornellBoxScene;
impl CornellBoxScene {
    pub fn insert_into(scene: &mut impl SceneT) {
        let default_material = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(Gooch {
                diffuse: Rgb::from_array([1.0, 0., 0.]),
                smoothness: 20.0,
                light_dir: Vec3::new(-1.0, -1.0, 0.0),
                yellow: Rgb::from_array([0.8, 0.8, 0.0]),
                blue: Rgb::from_array([0.0, 0.0, 0.8]),
            }),
        });

        scene.load_obj(
            "./obj/cornell_box.obj",
            Transform {
                translation: Vec3::new(0.0, -0.5, -0.5),
                scale: Vec3::splat(0.5),
                rot: Quat::IDENTITY,
            },
            default_material,
        );

        scene.insert_light(LightDescriptor {
            label: None,
            light_pos: Point::new(0.0, 0.4, -0.4),
        });
    }
}
