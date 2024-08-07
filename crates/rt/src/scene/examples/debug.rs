use crate::{material::DiffuseBxDF, math::point::Point, scene::SceneT};

pub struct DebugScene;

impl DebugScene {
    pub fn insert_into<S: SceneT>(scene: &mut S) {
        let default_material = scene.insert_material(crate::material::MaterialDescriptor {
            label: Some("Goosh - Default".to_string()),
            material: Box::new(DiffuseBxDF {
                albedo: [1.0, 1.0, 0.0].into(),
            }),
        });

        scene.insert_sphere(default_material, Point::new(0.0, 0.0, -1.0), 0.3);
        scene.insert_sphere(default_material, Point::new(0.5, 0.0, -1.0), 0.3);
    }
}
