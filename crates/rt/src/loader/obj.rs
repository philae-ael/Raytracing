use std::path::PathBuf;

use crate::{
    aggregate::{
        embree::EmbreeScene,
        shapelist::{ShapeList, ShapeListEntry},
    },
    color::Rgb,
    material::{texture, Diffuse, Emit, MaterialId, MixMaterial},
    math::{
        point::Point,
        transform::{Transform, Transformer},
    },
    scene::Scene,
    shape::TriangleBuilder,
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
                    x.split(' ')
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

                let mat_id = if let Some(ke) = ke {
                    self.insert_material(
                        None,
                        MixMaterial {
                            p: 0.5,
                            mat1: Diffuse {
                                texture: Box::new(texture::Uniform(Rgb::from_array(
                                    material.diffuse,
                                ))),
                            },
                            mat2: Emit {
                                texture: Box::new(texture::Uniform(Rgb::from_array(ke))),
                            },
                        },
                    )
                } else {
                    self.insert_material(
                        None,
                        Diffuse {
                            texture: Box::new(texture::Uniform(Rgb::from_array(material.diffuse))),
                        },
                    )
                };

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

            let mut triangles: ShapeList = Default::default();

            log::debug!("indices: {:?}", mesh.indices);
            // TODO: Grab normals if any
            let mut indices_slice = mesh.indices.as_slice();
            for _ in 0..num_faces {
                let indices = &indices_slice[0..3];
                indices_slice = &indices_slice[3..];

                #[allow(clippy::identity_op)]
                let vertices = [
                    Point::new(
                        mesh.positions[(0 + indices[0] * 3) as usize],
                        mesh.positions[(1 + indices[0] * 3) as usize],
                        mesh.positions[(2 + indices[0] * 3) as usize],
                    ),
                    Point::new(
                        mesh.positions[(0 + indices[1] * 3) as usize],
                        mesh.positions[(1 + indices[1] * 3) as usize],
                        mesh.positions[(2 + indices[1] * 3) as usize],
                    ),
                    Point::new(
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

                triangles.0.push(ShapeListEntry::Shape(Box::new(
                    TriangleBuilder {
                        vertices,
                        winding: Default::default(),
                    }
                    .build(material),
                )));
            }

            self.insert_shape_list(triangles)
        }
    }
}

impl ObjLoaderExt for EmbreeScene {
    fn load_obj<T: Into<PathBuf>>(&mut self, mesh_path: T, transform: Transform, _: MaterialId) {
        let mut options = tobj::GPU_LOAD_OPTIONS;
        options.single_index = true;
        let (models, _) =
            tobj::load_obj(mesh_path.into(), &options).expect("Failed to load OBJ file");

        let mut voutput = Vec::<(f32, f32, f32)>::new();
        let mut ioutput = Vec::new();

        for model in models {
            let mesh = model.mesh;
            let num_faces = mesh.indices.len() / 3;
            log::debug!("Loading model {}; {} faces", model.name, num_faces);

            log::debug!("indices: {:?}", mesh.indices);
            let mut indices_slice = mesh.indices.as_slice();
            for _ in 0..num_faces {
                let indices = &indices_slice[0..3];
                indices_slice = &indices_slice[3..];

                #[allow(clippy::identity_op)]
                let vertices = [
                    Point::new(
                        mesh.positions[(0 + indices[0] * 3) as usize],
                        mesh.positions[(1 + indices[0] * 3) as usize],
                        mesh.positions[(2 + indices[0] * 3) as usize],
                    ),
                    Point::new(
                        mesh.positions[(0 + indices[1] * 3) as usize],
                        mesh.positions[(1 + indices[1] * 3) as usize],
                        mesh.positions[(2 + indices[1] * 3) as usize],
                    ),
                    Point::new(
                        mesh.positions[(0 + indices[2] * 3) as usize],
                        mesh.positions[(1 + indices[2] * 3) as usize],
                        mesh.positions[(2 + indices[2] * 3) as usize],
                    ),
                ]
                .map(|v| transform.apply(v));

                let i1 = voutput.len() as u32;
                voutput.push(vertices[0].0.to_array().into());
                voutput.push(vertices[1].0.to_array().into());
                voutput.push(vertices[2].0.to_array().into());

                ioutput.push((i1, i1 + 1, i1 + 2));

                log::debug!("Face for indices {:?} {vertices:?}", indices);
            }
        }
        self.attach_geometry(&voutput, &ioutput);
    }
}
