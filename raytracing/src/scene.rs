use glam::{Quat, Vec3};
use image::Rgb;

use crate::{
    aggregate::shapelist::ShapeList,
    loader::ObjLoaderExt,
    material::{texture, Emit, Gooch, MaterialDescriptor, MaterialId},
    math::transform::Transform,
    shape::Shape,
};

pub struct DefaultScene;

#[derive(Default)]
pub struct Scene {
    pub objects: ShapeList,
    pub materials: Vec<MaterialDescriptor>,
}

impl Scene {
    /// Insert an object in the scene
    pub fn insert_object<T: Shape + Sync + 'static>(&mut self, object: T) {
        self.objects.0.push(Box::new(object))
    }

    /// Insert a material and returns the Material ID associated with this material
    pub fn insert_material(&mut self, material: MaterialDescriptor) -> MaterialId {
        self.materials.push(material);
        MaterialId(self.materials.len() - 1)
    }
}

impl From<DefaultScene> for Scene {
    fn from(_: DefaultScene) -> Self {
        let mut scene = Scene::default();

        let default_material = scene.insert_material(MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(Gooch {
                diffuse: Rgb([1.0, 0., 0.]),
                smoothness: 20.0,
                light_dir: Vec3::new(-1.0, -1.0, 0.0),
                yellow: Rgb([0.8, 0.8, 0.0]),
                blue: Rgb([0.0, 0.0, 0.8]),
            }),
        });

        let _sky_material = scene.insert_material(MaterialDescriptor {
            label: Some("Sky".to_string()),
            material: Box::new(Emit {
                texture: Box::new(texture::Uniform(Rgb([0.5, 0.7, 0.9]))),
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

        // let light_mat = scene.insert_material(MaterialDescriptor {
        //     label: Some("Light".to_owned()),
        //     material: Box::new(Diffuse {
        //         texture: Box::new(texture::Uniform(Rgb([0.8, 0.8, 1.0]))),
        //     }),
        // });
        // scene.insert_object(Sphere {
        //     center: Vec3::new(0.5, 0.0, -1.0),
        //     radius: 0.01,
        //     material: light_mat,
        // });
        scene
    }
}
