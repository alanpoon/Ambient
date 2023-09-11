use ambient_api::{
    core::{prefab::components::prefab_from_url, transform::concepts::Transformable},
    prelude::*,
};

use packages::{
    orbit_camera::concepts::{OrbitCamera, OrbitCameraOptional},
    this::assets,
};

#[main]
pub async fn main() {
    // Camera
    OrbitCamera {
        is_orbit_camera: (),
        lookat_target: Vec3::ZERO,
        optional: OrbitCameraOptional {
            camera_distance: Some(2.0),
            ..default()
        },
    }
    .make()
    .spawn();

    // Model
    Entity::new()
        .with_merge(Transformable {
            local_to_world: Default::default(),
            optional: Default::default(),
        })
        .with(prefab_from_url(), assets::url("Cube.glb"))
        .spawn();
}
