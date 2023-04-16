use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use ambient_api::{
    components::core::{
        app::main_scene,
        model::model_from_url,
        physics::{
            angular_velocity, box_collider, density, dynamic, linear_velocity, physics_controlled,
            plane_collider,
        },
        player::player as player_component,
        rendering::{cast_shadows, fog_density, light_diffuse, sky, sun, water},
        transform::{rotation, scale, translation},
    },
    concepts::make_transformable,
    messages::Frame,
    prelude::*,
};

// How long a full cycle takes.
const HALF_DAY_LENGTH: f32 = 30.0;

const X_DISTANCE: f32 = 0.1;
const Y_DISTANCE: f32 = 0.4;
const OFFSETS: [(f32, f32); 4] = [
    (-X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, -Y_DISTANCE),
    (X_DISTANCE, Y_DISTANCE),
    (-X_DISTANCE, Y_DISTANCE),
];

const K_P: f32 = 150.0;
const K_D: f32 = -400.0;
const TARGET: f32 = 3.0;
const MAX_STRENGTH: f32 = 10.0;

const INPUT_FORWARD_FORCE: f32 = 20.0;
const INPUT_BACKWARD_FORCE: f32 = -4.0;
const INPUT_SIDE_FORCE: f32 = 0.8;

const DENSITY: f32 = 10.0;
const SLOWDOWN_STRENGTH: f32 = 0.75;

const ANGULAR_SLOWDOWN_DELAY: f32 = 0.25;
const ANGULAR_SLOWDOWN_STRENGTH: f32 = 0.2;

#[main]
pub fn main() {
    make_water();
    make_sun();

    vehicle_creation_and_destruction();
    vehicle_processing();
}

fn make_water() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(water())
        .with_default(physics_controlled())
        .with_default(plane_collider())
        .with(dynamic(), false)
        .with(scale(), Vec3::ONE * 2000.)
        .spawn();
}

fn make_sun() {
    Entity::new()
        .with_merge(make_transformable())
        .with_default(sky())
        .spawn();

    let sun = Entity::new()
        .with_merge(make_transformable())
        .with_default(sun())
        .with_default(rotation())
        .with_default(main_scene())
        .with(light_diffuse(), Vec3::ONE)
        .with(fog_density(), 0.0)
        .spawn();

    Frame::subscribe(move |_| {
        entity::set_component(
            sun,
            rotation(),
            Quat::from_rotation_y(PI + PI * (time() * PI / HALF_DAY_LENGTH).sin().abs()),
        );
    });
}

fn vehicle_creation_and_destruction() {
    spawn_query(player_component()).bind(|players| {
        for (player_id, ()) in players {
            let vehicle_id = Entity::new()
                .with_merge(make_transformable())
                .with_default(cast_shadows())
                .with_default(linear_velocity())
                .with_default(angular_velocity())
                .with_default(physics_controlled())
                .with(dynamic(), true)
                .with(components::vehicle(), player_id)
                .with(translation(), vec3(0., 0., 2.0))
                .with(density(), DENSITY)
                .with(components::last_distances(), OFFSETS.map(|_| 0.0).to_vec())
                .with(components::debug_messages(), vec![])
                .with(components::debug_lines(), vec![])
                .with(
                    model_from_url(),
                    asset::url("assets/models/dynamic/raceCarWhite.glb/models/main.json").unwrap(),
                )
                .with(box_collider(), Vec3::new(0.6, 1.0, 0.2))
                .spawn();
            entity::add_component(player_id, components::player_vehicle(), vehicle_id);
            entity::add_component(player_id, components::input_direction(), Vec2::ZERO);
            entity::add_component(player_id, components::input_jump(), false);
            entity::add_component(player_id, components::input_reset(), false);
        }
    });

    despawn_query(player_component()).bind(|players| {
        for (player, ()) in players {
            if let Some(vehicle) = entity::get_component(player, components::player_vehicle()) {
                entity::despawn(vehicle);
            }
        }
    });

    messages::Input::subscribe(|source, input| {
        if let Some(player) = source.client_entity_id() {
            entity::set_component(player, components::input_direction(), input.direction);
            entity::set_component(player, components::input_jump(), input.jump);
            entity::set_component(player, components::input_reset(), input.reset);
        }
    });
}

fn vehicle_processing() {
    let last_slowdown = Rc::new(RefCell::new(time()));

    query(components::vehicle()).each_frame(move |vehicles| {
        for (vehicle_id, driver_id) in vehicles {
            let freeze =
                entity::get_component(driver_id, components::input_jump()).unwrap_or_default();

            let direction =
                entity::get_component(driver_id, components::input_direction()).unwrap_or_default();

            let vehicle_position = match entity::get_component(vehicle_id, translation()) {
                Some(vehicle_position) => vehicle_position,
                _ => {
                    continue;
                }
            };
            let vehicle_rotation = match entity::get_component(vehicle_id, rotation()) {
                Some(vehicle_rotation) => vehicle_rotation,
                _ => {
                    continue;
                }
            };

            let mut last_distances =
                entity::get_component(vehicle_id, components::last_distances()).unwrap();

            for (index, offset) in OFFSETS.iter().enumerate() {
                let offset = Vec2::from(*offset).extend(0.0);

                let probe_start = vehicle_position + vehicle_rotation * (offset - Vec3::Z * 0.1);
                let probe_direction = vehicle_rotation * Vec3::Z * -1.0;

                if probe_direction.z > 0.0 {
                    continue;
                }

                if let Some(hit) = physics::raycast(probe_start, probe_direction)
                    .into_iter()
                    .find(|h| h.entity != vehicle_id)
                {
                    let old_distance = last_distances[index];
                    let new_distance = hit.distance;
                    let delta_distance = new_distance - old_distance;

                    let error_distance = TARGET - hit.distance;
                    let p = K_P * error_distance;
                    let d = K_D * delta_distance;
                    let strength = ((p + d) * frametime()).clamp(-0.1, MAX_STRENGTH);

                    let force = -probe_direction * strength;
                    let position = vehicle_position + vehicle_rotation * offset;
                    if !freeze {
                        physics::add_force_at_position(vehicle_id, force, position);
                    }

                    last_distances[index] = new_distance;
                }
            }
            entity::set_component(vehicle_id, components::last_distances(), last_distances);

            physics::add_force_at_position(
                vehicle_id,
                vehicle_rotation
                    * (Vec3::Y * direction.y.abs())
                    * -if direction.y > 0. {
                        INPUT_FORWARD_FORCE
                    } else {
                        INPUT_BACKWARD_FORCE
                    },
                vehicle_position + vehicle_rotation * Y_DISTANCE * Vec3::Y,
            );

            physics::add_force_at_position(
                vehicle_id,
                vehicle_rotation * (Vec3::X * -direction.x) * INPUT_SIDE_FORCE,
                vehicle_position + vehicle_rotation * -Y_DISTANCE * Vec3::Y,
            );

            if entity::get_component(driver_id, components::input_reset()).unwrap_or_default() {
                entity::set_component(vehicle_id, translation(), Vec3::Z * 7.0);
                entity::set_component(vehicle_id, rotation(), Quat::IDENTITY);
            }

            if freeze {
                entity::set_component(vehicle_id, linear_velocity(), Vec3::ZERO);
                entity::set_component(vehicle_id, angular_velocity(), Vec3::ZERO);
            }

            // Apply a constant slowdown force
            physics::add_force(
                vehicle_id,
                -entity::get_component(vehicle_id, linear_velocity()).unwrap_or_default()
                    * SLOWDOWN_STRENGTH,
            );

            if time() - *last_slowdown.borrow() > ANGULAR_SLOWDOWN_DELAY {
                entity::mutate_component(vehicle_id, angular_velocity(), |av| {
                    *av -= *av * ANGULAR_SLOWDOWN_STRENGTH;
                });
                *last_slowdown.borrow_mut() = time();
            }
        }
    });
}
