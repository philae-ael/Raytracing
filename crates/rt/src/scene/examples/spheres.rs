use glam::Vec3;

use crate::{
    color::Rgb,
    material::{texture, Dielectric, Diffuse, Emit},
    math::{point::Point, vec::Vec3AsRgbExt},
    scene::Scene,
    shape::{Plane, Sphere},
};

pub struct SpheresScene;
impl From<SpheresScene> for Scene {
    fn from(_: SpheresScene) -> Self {
        let mut scene = Scene::new(Emit {
            texture: Box::new(texture::Uniform(Rgb::from_array([0.01, 0.01, 0.01]))),
        });

        let diffuse = scene.insert_material(
            None,
            Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.9, 0.7]))),
            },
        );
        let diffuse_blue = scene.insert_material(
            None,
            Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.4, 0.8]))),
            },
        );
        let diffuse_ground = scene.insert_material(
            None,
            Diffuse {
                texture: Box::new(texture::Uniform(Rgb::from_array([0.2, 0.3, 0.7]))),
            },
        );
        let glass = scene.insert_material(
            None,
            Dielectric {
                texture: Box::new(texture::Uniform(Rgb::from_array([1.0, 1.0, 1.0]))),
                ior: 1.5,
            },
        );
        let light = scene.insert_material(
            None,
            Emit {
                texture: Box::new(texture::Uniform(Vec3::splat(15.0).rgb())),
            },
        );

        scene.insert_object(Sphere {
            center: Point::new(-0.6, 0.05, -1.0),
            radius: 0.3,
            material: diffuse,
        });
        scene.insert_object(Sphere {
            center: Point::new(-0.3, -0.05, 1.0),
            radius: 0.2,
            material: diffuse_blue,
        });
        scene.insert_object(Sphere {
            center: Point::new(0.0, 0.0, -0.3),
            radius: 0.15,
            material: glass,
        });
        // scene.insert_object(Plane {
            // origin: Point::new(0.0, 0.5, 0.0),
            // normal: -Vec3::Y,
            // material: diffuse_ground,
        // });
        scene.insert_object(Plane {
            origin: Point::new(0.0, -0.15, 0.0),
            normal: Vec3::Y,
            material: diffuse_ground,
        });

        scene.insert_light(Point::new(0.0, 0., -0.5));
        scene.insert_object(Sphere {
            center: Point::new(0.4, -0., -0.6),
            radius: 0.12,
            material: light,
        });
        scene.insert_object(Sphere {
            center: Point::new(-0.1, -0.1, 0.6),
            radius: 0.12,
            material: light,
        });
        scene
    }
}
