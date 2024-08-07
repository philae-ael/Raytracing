use glam::{Quat, Vec3};

use crate::{
    loader::ObjLoaderExt,
    material::{DiffuseBxDF, MaterialDescriptor},
    math::transform::Transform,
    scene::SceneT,
};

pub struct StandfordBunnyScene;
impl StandfordBunnyScene {
    pub fn insert_into(scene: &mut impl SceneT) {
        let default_material = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(DiffuseBxDF {
                albedo: [1.0, 1.0, 0.5].into(),
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
    }
}
