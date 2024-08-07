use std::{collections::BTreeMap, mem::size_of};

use anyhow::Result;
use embree4_rs::{
    device::Device,
    geometry::{Geometry, SphereGeometry},
    scene::{CommittedScene, Scene, SceneOptions},
};
use embree4_sys::{RTCGeometry, RTCSceneFlags};

use crate::{
    material::{DiffuseBxDF, MaterialDescriptor, MaterialId},
    math::point::Point,
    renderer::World,
    scene::SceneT,
    shape::{local_info, FullIntersectionResult, Shape},
};

pub struct EmbreeScene<'a> {
    device: &'a Device,
    scene: Scene<'a>,
    pub materials: Vec<MaterialDescriptor>,
    pub lights: Vec<Point>,
    pub geometry_material: BTreeMap<<Self as SceneT>::GeometryHandle, MaterialId>,
    sky_material: MaterialId,
}

impl<'a> EmbreeScene<'a> {
    pub fn new(device: &'a Device) -> Self {
        let scene = Scene::try_new(
            device,
            SceneOptions {
                build_quality: embree4_sys::RTCBuildQuality::HIGH,
                flags: RTCSceneFlags::ROBUST,
            },
        )
        .unwrap();
        Self {
            device,
            scene,
            materials: vec![MaterialDescriptor {
                label: Some("Sky".into()),
                material: Box::new(DiffuseBxDF {
                    albedo: [0.0, 0.0, 0.0].into(),
                }),
            }],
            lights: Default::default(),
            geometry_material: Default::default(),
            sky_material: MaterialId(0),
        }
    }

    pub fn insert_geometry(
        &mut self,
        mat: MaterialId,
        geom: &impl Geometry,
    ) -> <Self as SceneT>::GeometryHandle {
        let geom_id = self.scene.attach_geometry(geom).unwrap();
        self.geometry_material.insert(geom_id, mat);
        geom_id
    }

    pub fn commit<'c>(&'c mut self) -> Result<CommittedEmbreeScene<'c, 'a>> {
        let commited = self.scene.commit()?;
        Ok(CommittedEmbreeScene {
            scene: self,
            commited,
        })
    }
    pub fn commit_with_progress<'c, F: FnMut(f64) -> bool>(
        &'c mut self,
        progress_callback: F,
    ) -> Result<CommittedEmbreeScene<'c, 'a>> {
        let _p = self
            .scene
            .register_scene_progress_monitor_callback(progress_callback);
        self.commit()
    }
}

pub struct CommittedEmbreeScene<'a, 'b> {
    scene: &'a EmbreeScene<'b>,
    commited: CommittedScene<'b>,
}

unsafe impl Send for CommittedEmbreeScene<'_, '_> {}
unsafe impl Sync for CommittedEmbreeScene<'_, '_> {}

impl<'a, 'b> CommittedEmbreeScene<'a, 'b> {
    pub fn into_world(&self) -> Result<World> {
        Ok(World {
            objects: self,
            lights: &self.scene.lights,
            materials: &self.scene.materials,
            world_material: self.scene.sky_material,
        })
    }
}

impl Shape for CommittedEmbreeScene<'_, '_> {
    fn intersection_full(&self, ray: crate::ray::Ray) -> crate::shape::FullIntersectionResult {
        let r = embree4_sys::RTCRay {
            org_x: ray.origin.0.x,
            org_y: ray.origin.0.y,
            org_z: ray.origin.0.z,
            dir_x: ray.direction.x,
            dir_y: ray.direction.y,
            dir_z: ray.direction.z,
            tnear: ray.bounds.0,
            tfar: ray.bounds.1,
            ..Default::default()
        };

        match self.commited.intersect_1(r).unwrap() {
            Some(res) => FullIntersectionResult::Intersection(crate::shape::RayIntersection {
                t: res.ray.tfar,
                local_info: local_info::Full {
                    pos: Point::new(
                        res.ray.org_x + res.ray.tfar * res.ray.dir_x,
                        res.ray.org_y + res.ray.tfar * res.ray.dir_y,
                        res.ray.org_z + res.ray.tfar * res.ray.dir_z,
                    ),
                    normal: glam::Vec3 {
                        x: res.hit.Ng_x,
                        y: res.hit.Ng_y,
                        z: res.hit.Ng_z,
                    }
                    .normalize_or_zero(),
                    material: self
                        .scene
                        .geometry_material
                        .get(&res.hit.geomID)
                        .copied()
                        .unwrap_or(MaterialId(0)),
                    uv: [res.hit.u, res.hit.v],
                },
            }),
            None => FullIntersectionResult::NoIntersection,
        }
    }

    fn intersect_bare(&self, _ray: crate::ray::Ray) -> crate::shape::MinIntersectionResult {
        todo!()
    }

    fn bounding_box(&self) -> crate::math::bounds::Bounds {
        let bounding = self.commited.bounds().unwrap();

        crate::math::bounds::Bounds {
            origin: crate::math::point::Point::new(
                bounding.lower_x,
                bounding.lower_y,
                bounding.lower_z,
            ),
            end: crate::math::point::Point::new(
                bounding.upper_x,
                bounding.upper_y,
                bounding.upper_z,
            ),
        }
    }
}

impl SceneT for EmbreeScene<'_> {
    type GeometryHandle = u32;

    fn insert_material(&mut self, mat: crate::material::MaterialDescriptor) -> MaterialId {
        let mat_id = MaterialId(self.materials.len());
        self.materials.push(mat);
        mat_id
    }

    fn insert_light(&mut self, light: crate::material::LightDescriptor) {
        self.lights.push(light.light_pos);
    }

    fn insert_mesh(
        &mut self,
        material: MaterialId,
        vertices: &[[f32; 3]],
        indices: &[[u32; 3]],
    ) -> Self::GeometryHandle {
        let geometry = {
            let geometry = unsafe {
                embree4_sys::rtcNewGeometry(
                    self.device.as_raw_handle(),
                    embree4_sys::RTCGeometryType::TRIANGLE,
                )
            };
            if geometry.is_null() {
                panic!("Failed to create geometry: {:?}", self.device.error());
            }

            let vertex_buf_ptr = unsafe {
                embree4_sys::rtcSetNewGeometryBuffer(
                    geometry,
                    embree4_sys::RTCBufferType::VERTEX,
                    0,
                    embree4_sys::RTCFormat::FLOAT3,
                    3 * size_of::<f32>(),
                    vertices.len(),
                )
            };
            if vertex_buf_ptr.is_null() {
                panic!(
                    "Failed to create triangle mesh vertex buffer: {:?}",
                    self.device.error()
                );
            }
            if let Some(err) = self.device.error() {
                panic!("{:?}", err);
            }

            unsafe {
                std::slice::from_raw_parts_mut(vertex_buf_ptr as *mut f32, 3 * vertices.len())
            }
            .copy_from_slice(bytemuck::cast_slice(vertices));

            let index_buf_ptr = unsafe {
                embree4_sys::rtcSetNewGeometryBuffer(
                    geometry,
                    embree4_sys::RTCBufferType::INDEX,
                    0,
                    embree4_sys::RTCFormat::UINT3,
                    3 * size_of::<u32>(),
                    indices.len(),
                )
            };
            if index_buf_ptr.is_null() {
                panic!(
                    "Failed to create triangle mesh index buffer: {:?}",
                    self.device.error()
                );
            }

            if let Some(err) = self.device.error() {
                panic!("Failed to create triangle mesh index buffer {:?}", err);
            }

            unsafe { std::slice::from_raw_parts_mut(index_buf_ptr as *mut u32, 3 * indices.len()) }
                .copy_from_slice(bytemuck::cast_slice(indices));

            unsafe {
                embree4_sys::rtcCommitGeometry(geometry);
            }
            if let Some(err) = self.device.error() {
                panic!("Failed to create triangle mesh index buffer {:?}", err);
            }

            CustomGeometry { handle: geometry }
        };
        self.insert_geometry(material, &geometry)
    }

    fn insert_sphere(
        &mut self,
        material: MaterialId,
        center: Point,
        radius: f32,
    ) -> Self::GeometryHandle {
        let geom =
            SphereGeometry::try_new(self.device, (center.0.x, center.0.y, center.0.z), radius)
                .unwrap();
        self.insert_geometry(material, &geom)
    }
}

struct CustomGeometry {
    pub handle: RTCGeometry,
}

impl Drop for CustomGeometry {
    fn drop(&mut self) {
        unsafe {
            embree4_sys::rtcReleaseGeometry(self.handle);
        }
    }
}

impl Geometry for CustomGeometry {
    fn geometry(&self) -> embree4_sys::RTCGeometry {
        self.handle
    }
}
