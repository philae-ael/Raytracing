use glam::Vec3;

use crate::{
    color::Rgb,
    material::{texture, Dielectric, Diffuse, Emit, LightDescriptor, MaterialDescriptor},
    math::{point::Point, vec::Vec3AsRgbExt},
    scene::SceneT,
};

pub struct SpheresScene;

impl SpheresScene {
    pub fn insert_into<S: SceneT>(scene: &mut S) {
        let diffuse = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.9, 0.7]))),
            }),
        });
        let diffuse_blue = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.4, 0.8]))),
            }),
        });
        let glass = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Dielectric {
                texture: Box::new(texture::Uniform(Rgb::from_array([1.0, 1.0, 1.0]))),
                ior: 1.5,
            }),
        });
        let light = scene.insert_material(MaterialDescriptor {
            label: None,
            material: Box::new(Emit {
                texture: Box::new(texture::Uniform(Vec3::splat(15.0).rgb())),
            }),
        });

        scene.insert_sphere(diffuse, Point::new(-0.6, 0.05, -1.0), 0.3);
        scene.insert_sphere(diffuse_blue, Point::new(-0.3, -0.05, 1.0), 0.2);
        scene.insert_sphere(glass, Point::new(0.0, 0.0, -0.3), 0.15);

        // let diffuse_ground = scene.insert_material(MaterialDescriptor {
        //     label: None,
        //     material: Box::new(Diffuse {
        //         texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.3, 0.7]))),
        //     }),
        // });
        // scene.insert_object(Plane {
        // origin: Point::new(0.0, 0.5, 0.0),
        // normal: -Vec3::Y,
        // material: diffuse_ground,
        // });
        // scene.insert_plane(diffuse_ground, Point::new(0.0, -0.15, 0.0), Vec3::Y);

        scene.insert_light(LightDescriptor {
            label: None,
            light_pos: Point::new(0.0, 0., -0.5),
        });
        scene.insert_sphere(light, Point::new(0.4, -0., -0.6), 0.12);
        scene.insert_sphere(light, Point::new(-0.1, -0.1, 0.6), 0.12);
    }
}
