use embree4_rs::{geometry::TriangleMeshGeometry, CommittedScene, Device, Scene, SceneOptions};
use embree4_sys::RTCSceneFlags;

use crate::{
    material::MaterialId,
    math::point::Point,
    shape::{local_info, FullIntersectionResult, Shape},
};

pub struct EmbreeScene {
    scene: Scene,
}

impl EmbreeScene {
    pub fn new(dev: Device) -> Self {
        let scene = Scene::try_new(
            dev,
            SceneOptions {
                build_quality: embree4_sys::RTCBuildQuality::HIGH,
                flags: RTCSceneFlags::ROBUST,
            },
        )
        .unwrap();
        Self { scene }
    }
    pub fn attach_geometry(&mut self, vertices: &[(f32, f32, f32)], indices: &[(u32, u32, u32)]) {
        let mesh = TriangleMeshGeometry::try_new(&self.scene.device, vertices, indices).unwrap();
        self.scene.attach_geometry(&mesh).unwrap();
    }

    pub fn commit(self) -> CommittedEmbreeScene {
        CommittedEmbreeScene {
            scene: self.scene.commit().unwrap(),
        }
    }
}

pub struct CommittedEmbreeScene {
    scene: CommittedScene,
}

unsafe impl Send for CommittedEmbreeScene {}

impl Shape for CommittedEmbreeScene {
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

        match self.scene.intersect_1(r, None).unwrap() {
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
                    material: MaterialId(0),
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
        let bounding = self.scene.bounds().unwrap();

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
