use glam::{Quat, Vec3};

use crate::{
    color::Rgb,
    loader::ObjLoaderExt,
    material::{texture, Emit, Gooch},
    math::{point::Point, transform::Transform},
    scene::Scene,
};

pub struct CornellBoxScene;
impl From<CornellBoxScene> for Scene {
    fn from(_: CornellBoxScene) -> Self {
        let mut scene = Scene::new(Emit {
            texture: Box::new(texture::Uniform(Rgb::from_array([0.0, 0.0, 0.0]))),
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

        scene.load_obj(
            "./obj/cornell_box.obj",
            Transform {
                translation: Vec3::new(0.0, -0.5, -0.5),
                scale: Vec3::splat(0.5),
                rot: Quat::IDENTITY,
            },
            default_material,
        );

        scene.insert_light(Point::new(0.0, 0.4, -0.4));

        scene
    }
}
