use glam::Vec3;
use image::Rgb;

use crate::{
    aggregate::shapelist::ShapeList,
    material::{texture, Dielectric, Diffuse, Emit, MaterialDescriptor, MaterialId, Metal},
    shape::Sphere,
};

pub struct DefaultScene;

pub struct Scene {
    pub objects: ShapeList,
    pub materials: Vec<MaterialDescriptor>,
}

impl Into<Scene> for DefaultScene {
    fn into(self) -> Scene {
        let materials: Vec<MaterialDescriptor> = vec![
            MaterialDescriptor {
                label: Some("Bubble like".to_string()),
                material: Box::new(Dielectric {
                    texture: Box::new(texture::Uniform(Rgb([0.7, 0.3, 0.3]))),
                    ior: 0.8,
                    invert_normal: false,
                }),
            },
            MaterialDescriptor {
                label: Some("Diffuse orange".to_string()),
                material: Box::new(Diffuse {
                    texture: Box::new(texture::Uniform(Rgb([0.8, 0.6, 0.2]))),
                }),
            },
            MaterialDescriptor {
                label: Some("Gray metal".to_string()),
                material: Box::new(Metal {
                    texture: Box::new(texture::Uniform(Rgb([0.8, 0.8, 0.8]))),
                    roughness: 0.6,
                }),
            },
            MaterialDescriptor {
                label: Some("Ground".to_string()),
                material: Box::new(Diffuse {
                    texture: Box::new(texture::Uniform(Rgb([0.2, 0.4, 0.3]))),
                }),
            },
            MaterialDescriptor {
                label: Some("Light".to_string()),
                material: Box::new(Emit {
                    texture: Box::new(texture::Uniform(Rgb([2.5, 3.7, 3.9]))),
                }),
            },
            MaterialDescriptor {
                label: Some("Sky".to_string()),
                material: Box::new(Emit {
                    texture: Box::new(texture::Uniform(Rgb([0.5, 0.8, 0.5]))),
                }),
            },
        ];

        let objects = ShapeList(vec![
            Box::new(Sphere {
                center: Vec3::new(0.0, 0.0, -1.),
                radius: 0.5,
                material: MaterialId(0),
            }),
            Box::new(crate::shape::implicit::ImplicitShape {
                surface: crate::shape::implicit::Cube {
                    origin: Vec3::new(1.0, 0.0, -1.),
                    size: 0.5,
                },
                solver: crate::shape::implicit::solvers::NewtonSolver {
                    max_iter: 2,
                    eps: 0.01,
                },
                material: MaterialId(1),
            }),
            Box::new(Sphere {
                center: Vec3::new(-1.0, 0.0, -1.),
                radius: 0.5,
                material: MaterialId(2),
            }),
            Box::new(Sphere {
                center: Vec3::new(0.0, -100.5, -1.),
                radius: 100.,
                material: MaterialId(3),
            }),
            Box::new(Sphere {
                center: Vec3::new(0.5, -0.4, -0.5),
                radius: 0.1,
                material: MaterialId(4),
            }),
        ]);

        return Scene { objects, materials };
    }
}
