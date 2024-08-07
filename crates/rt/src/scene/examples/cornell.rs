use crate::material::{texture, BSDFMaterial, Diffuse, DiffuseBxDF};
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
        let c = Box::new(texture::Uniform(Rgb::from_array([0.5, 0.8, 0.9])));
        let default_material = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(Diffuse { texture: c }),
        });

        let default_material2 = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default 2".to_string()),
            material: Box::new(BSDFMaterial {
                bxdf: DiffuseBxDF {
                    albedo: Rgb::from_array([0.5, 0.8, 0.9]),
                },
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
    }
}
