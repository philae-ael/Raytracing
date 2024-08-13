use crate::material::{DiffuseBxDF, EmitBxDF};
use crate::scene::SceneT;
use crate::{
    color::Rgb,
    loader::ObjLoaderExt,
    material::{LightDescriptor, MaterialDescriptor},
    math::{point::Point, transform::Transform},
};
use glam::{Quat, Vec3};

pub struct CornellBoxScene;
impl CornellBoxScene {
    pub fn insert_into(scene: &mut impl SceneT) {
        let default_material2 = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default 2".to_string()),
            material: Box::new(DiffuseBxDF {
                albedo: Rgb::from_array([5.5, 0.8, 0.9]),
            }),
        });

        scene.load_obj(
            "./obj/cornell_box.obj",
            Transform {
                translation: Vec3::new(0.0, -0.5, -0.5),
                scale: Vec3::splat(0.5),
                rot: Quat::IDENTITY,
            },
            default_material2,
        );

        scene.insert_light(LightDescriptor {
            label: None,
            light_pos: Point::new(0.0, 0.4, -0.4),
        });
        let l = scene.insert_material(MaterialDescriptor {
            label: Some("light!".into()),
            material: Box::new(EmitBxDF {
                le: [5.0, 5.0, 5.0].into(),
            }),
        });
        scene.insert_sphere(l, Point::new(0.0, 0.0, 5.0), 3.0);
    }
}
