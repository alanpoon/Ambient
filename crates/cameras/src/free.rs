use ambient_core::{camera::*, transform::*};
use ambient_ecs::{components, query_mut, Entity, EntityId, SystemGroup};
use derive_more::Display;
use glam::vec2;
use winit::{event::{DeviceEvent, ElementState, Event, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};

use super::camera_movement_speed;

components!("camera", {
    free_camera: FreeCamera,
});

#[derive(Debug, Default, Display, Clone)]
#[display(fmt = "{self:?}")]
pub struct FreeCamera {
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    orientation: glam::Vec2,
}

pub fn new(position: glam::Vec3, orientation: glam::Vec2) -> Entity {
    let free = FreeCamera {
        orientation,
        ..Default::default()
    };
    Entity::new()
        .with(local_to_world(), Default::default())
        .with(inv_local_to_world(), Default::default())
        .with(near(), 0.1)
        .with(fovy(), 1.0)
        .with(perspective_infinite_reverse(), ())
        .with(aspect_ratio(), 1.)
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with(projection(), Default::default())
        .with(projection_view(), Default::default())
        .with(translation(), position)
        .with(rotation(), Default::default())
        .with(free_camera(), free)
        .with(camera_movement_speed(), 0.1)
}

pub fn free_camera_system() -> SystemGroup<Event<()>> {
    SystemGroup::new(
        "free_camera_system",
        vec![query_mut(
            (
                free_camera(),
                translation(),
                rotation(),
                camera_movement_speed(),
                far(),
            ),
            (),
        )
        .to_system(|q, world, qs, event| {
            for (_, (free_camera, translation, rotation, speed, far), ()) in q.iter(world, qs) {
                match event {
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        let speed = 0.01;
                        free_camera.orientation += vec2(delta.0 as f32, delta.1 as f32) * speed;
                    }
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput { event, .. },
                        ..
                    } => {
                        let is_pressed = event.state == ElementState::Pressed;

                        if let PhysicalKey::Code(keycode) = event.physical_key {
                            match keycode {
                                KeyCode::KeyE => free_camera.is_up_pressed = is_pressed,
                                KeyCode::KeyQ => free_camera.is_down_pressed = is_pressed,
                                KeyCode::KeyW | KeyCode::ArrowUp => {
                                    free_camera.is_forward_pressed = is_pressed
                                }
                                KeyCode::KeyA | KeyCode::ArrowLeft => {
                                    free_camera.is_left_pressed = is_pressed
                                }
                                KeyCode::KeyS | KeyCode::ArrowDown => {
                                    free_camera.is_backward_pressed = is_pressed
                                }
                                KeyCode::KeyD | KeyCode::ArrowRight => {
                                    free_camera.is_right_pressed = is_pressed
                                }
                                KeyCode::KeyR => *speed *= 2.0,
                                KeyCode::KeyF => *speed /= 2.0,
                                KeyCode::KeyT => *far *= 2.0,
                                KeyCode::KeyG => *far /= 2.0,
                                _ => {}
                            }
                        }
                    }
                    Event::WindowEvent {  event:WindowEvent::RedrawRequested,.. }=>{
                        let mut velocity = glam::Vec3::ZERO;
                        if free_camera.is_up_pressed {
                            velocity += glam::Vec3::Z;
                        }
                        if free_camera.is_down_pressed {
                            velocity -= glam::Vec3::Z;
                        }
                        if free_camera.is_forward_pressed {
                            velocity += (*rotation) * glam::Vec3::Z;
                        }
                        if free_camera.is_backward_pressed {
                            velocity -= (*rotation) * glam::Vec3::Z;
                        }
                        if free_camera.is_left_pressed {
                            velocity -= (*rotation) * glam::Vec3::X;
                        }
                        if free_camera.is_right_pressed {
                            velocity += (*rotation) * glam::Vec3::X;
                        }
                        *translation += velocity * (*speed);

                        *rotation = glam::Quat::from_rotation_z(free_camera.orientation.x)
                            * glam::Quat::from_rotation_x(free_camera.orientation.y);
                        // *rotation = glam::Quat::from_rotation_z(free_camera.orientation.x)
                        //     * glam::Quat::from_rotation_y(free_camera.orientation.y);
                    }
                    _ => {}
                }
            }
        })],
    )
}
