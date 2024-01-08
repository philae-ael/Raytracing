use glam::{Quat, Vec3};

use crate::{
    loader::ObjLoaderExt,
    material::{Gooch, MaterialDescriptor},
    math::transform::Transform,
    scene::Scene, color::Rgb,
};

pub struct StandfordBunnyScene;
impl From<StandfordBunnyScene> for Scene {
    fn from(_: StandfordBunnyScene) -> Self {
        let mut scene = Scene::default();

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
            "./obj/standford_bunny.obj",
            Transform {
                translation: Vec3::new(0.2, -0.3, -0.5),
                scale: Vec3::splat(4.0),
                rot: Quat::IDENTITY,
            },
            default_material,
        );

        scene
    }
}
