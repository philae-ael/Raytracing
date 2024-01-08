use std::path::PathBuf;

use glam::Vec3;

use crate::{
    aggregate::shapelist::ShapeList,
    color::Rgb,
    material::{texture, Diffuse, Emit, Material, MaterialDescriptor, MaterialId, MixMaterial},
    math::transform::Transform,
    scene::Scene,
    shape::{Shape, TriangleBuilder},
};

pub trait ObjLoaderExt {
    fn load_obj<T: Into<PathBuf>>(
        &mut self,
        mesh_path: T,
        transform: Transform,
        default_material: MaterialId,
    );
}

impl ObjLoaderExt for Scene {
    fn load_obj<T: Into<PathBuf>>(
        &mut self,
        mesh_path: T,
        transform: Transform,
        default_material: MaterialId,
    ) {
        let mut options = tobj::GPU_LOAD_OPTIONS;
        options.single_index = true;
        let (models, materials) =
            tobj::load_obj(mesh_path.into(), &options).expect("Failed to load OBJ file");

        // log::info!(
        //     "Found {} models and {} materials",
        //     models.len(),
        //     materials.unwrap_or(vec![]).len(),
        // );
        let mut material_ids = vec![];

        let has_non_default_materials = if let Ok(materials) = materials {
            for material in materials {
                let ke: Option<_> = material.unknown_param.get("Ke").and_then(|x| {
                    x.split(" ")
                        .map(|x| -> f32 { x.parse::<i32>().unwrap() as f32 })
                        .collect::<Vec<_>>()
                        .try_into()
                        .ok()
                        .and_then(|x: [f32; 3]| {
                            if x[0] == 0.0 && x[1] == 0.0 && x[2] == 0.0 {
                                None
                            } else {
                                Some(x)
                            }
                        })
                });

                let scene_mat: Box<dyn Material + Sync + Send> = if let Some(ke) = ke {
                    Box::new(MixMaterial {
                        p: 0.5,
                        mat1: Diffuse {
                            texture: Box::new(texture::Uniform(Rgb::from_array(material.diffuse))),
                        },
                        mat2: Emit {
                            texture: Box::new(texture::Uniform(Rgb::from_array(ke))),
                        },
                    })
                } else {
                    Box::new(Diffuse {
                        texture: Box::new(texture::Uniform(Rgb::from_array(material.diffuse))),
                    })
                };

                let mat_id = self.insert_material(MaterialDescriptor {
                    label: Some("()".to_owned()),
                    material: scene_mat,
                });

                log::debug!(
                    "Inserting material {} with diffuse {:?} on mat_id {:?}",
                    material.name,
                    material.diffuse,
                    mat_id
                );

                material_ids.push(mat_id);
            }
            true
        } else {
            false
        };

        for model in models {
            let mesh = model.mesh;
            let num_faces = mesh.indices.len() / 3;
            log::debug!("Loading model {}; {} faces", model.name, num_faces);

            let mut triangles: Vec<Box<dyn Shape + Sync + Send>> = Vec::new();

            log::debug!("indices: {:?}", mesh.indices);
            // TODO: Grab normals if any
            let mut indices_slice = mesh.indices.as_slice();
            for _ in 0..num_faces {
                let indices = &indices_slice[0..3];
                indices_slice = &indices_slice[3..];

                let vertices = [
                    Vec3::new(
                        mesh.positions[(0 + indices[0] * 3) as usize],
                        mesh.positions[(1 + indices[0] * 3) as usize],
                        mesh.positions[(2 + indices[0] * 3) as usize],
                    ),
                    Vec3::new(
                        mesh.positions[(0 + indices[1] * 3) as usize],
                        mesh.positions[(1 + indices[1] * 3) as usize],
                        mesh.positions[(2 + indices[1] * 3) as usize],
                    ),
                    Vec3::new(
                        mesh.positions[(0 + indices[2] * 3) as usize],
                        mesh.positions[(1 + indices[2] * 3) as usize],
                        mesh.positions[(2 + indices[2] * 3) as usize],
                    ),
                ]
                .map(|v| transform.apply(v));

                log::debug!("Face for indices {:?} {vertices:?}", indices);

                let material = if has_non_default_materials {
                    match mesh.material_id {
                        Some(mat_id) => *material_ids.get(mat_id).unwrap_or(&default_material),
                        None => default_material,
                    }
                } else {
                    default_material
                };

                triangles.push(
                    Box::new(
                        TriangleBuilder {
                            vertices,
                            ..Default::default()
                        }
                        .build(material),
                    ), // TODO: use material which is inside OBJ file
                );
            }
            self.insert_object(ShapeList(triangles));
        }
    }
}
