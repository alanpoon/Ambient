use ambient_ecs::{components, query_mut, Entity, SystemGroup};
use ambient_native_std::math::SphericalCoords;
use derive_more::Display;
use winit::{event::{
    DeviceEvent, ElementState, Event, MouseScrollDelta, WindowEvent,
}, keyboard::{Key, PhysicalKey}};
use winit::keyboard::KeyCode;
use super::*;

components!("camera", {
    spherical_camera: SphericalCamera,
});

#[derive(Debug, Default, Display, Clone)]
#[display(fmt = "{self:?}")]
pub struct SphericalCamera {
    is_rotating: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    orientation: SphericalCoords,
}
impl SphericalCamera {
    fn translation(&self, lookat_target: glam::Vec3) -> glam::Vec3 {
        lookat_target + glam::Vec3::from(self.orientation)
    }
}

pub fn new(lookat: glam::Vec3, orientation: SphericalCoords) -> Entity {
    let spherical = SphericalCamera {
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
        .with(translation(), spherical.translation(lookat))
        .with(lookat_target(), lookat)
        .with(lookat_up(), glam::vec3(0., 0., 1.))
        .with(spherical_camera(), spherical)
        .with(camera_movement_speed(), 0.1)
}

pub fn spherical_camera_system() -> SystemGroup<Event<()>> {
    SystemGroup::new(
        "spherical_camera_system",
        vec![query_mut(
            (
                spherical_camera(),
                translation(),
                lookat_target(),
                camera_movement_speed(),
            ),
            (),
        )
        .to_system(|q, world, qs, event| {
            for (_, (spherical_camera, translation, lookat_target, speed), ()) in q.iter(world, qs)
            {
                match event {
                    Event::DeviceEvent {
                        event: DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        if spherical_camera.is_rotating {
                            let speed = 0.01;
                            spherical_camera.orientation.phi += delta.0 as f32 * speed;
                            spherical_camera.orientation.theta -= delta.1 as f32 * speed;
                        }
                    }
                    Event::WindowEvent { event, .. } => {
                        match event {
                            WindowEvent::KeyboardInput { event, .. } => {
                                let is_pressed = event.state == ElementState::Pressed;
                                if let PhysicalKey::Code(keycode) = event.physical_key {
                                    match keycode {
                                        KeyCode::KeyE => {
                                            spherical_camera.is_up_pressed = is_pressed
                                        }
                                        KeyCode::KeyQ => {
                                            spherical_camera.is_down_pressed = is_pressed
                                        }
                                        KeyCode::KeyW | KeyCode::ArrowUp => {
                                            spherical_camera.is_forward_pressed = is_pressed
                                        }
                                        KeyCode::KeyA | KeyCode::ArrowLeft => {
                                            spherical_camera.is_left_pressed = is_pressed
                                        }
                                        KeyCode::KeyS | KeyCode::ArrowDown => {
                                            spherical_camera.is_backward_pressed = is_pressed
                                        }
                                        KeyCode::KeyD | KeyCode::ArrowRight => {
                                            spherical_camera.is_right_pressed = is_pressed
                                        }
                                        KeyCode::KeyR => *speed *= 2.0,
                                        KeyCode::KeyF => *speed /= 2.0,
                                        KeyCode::Space => {
                                            spherical_camera.is_rotating = is_pressed
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            WindowEvent::MouseWheel { delta, .. } => {
                                spherical_camera.orientation.radius *= 1.
                                    + match delta {
                                        MouseScrollDelta::LineDelta(_, y) => y * 0.05,
                                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.01,
                                    }
                            }
                            WindowEvent::MouseInput { .. } => {
                                // spherical_camera.is_rotating = state == &ElementState::Pressed;
                            }
                            WindowEvent::RedrawRequested{..} => {
                                let mut velocity = glam::Vec3::ZERO;
                                let rotation =
                                    glam::Quat::from_rotation_z(spherical_camera.orientation.phi);
                                if spherical_camera.is_up_pressed {
                                    velocity += glam::Vec3::Z;
                                }
                                if spherical_camera.is_down_pressed {
                                    velocity -= glam::Vec3::Z;
                                }
                                if spherical_camera.is_forward_pressed {
                                    velocity -= rotation * glam::Vec3::X;
                                }
                                if spherical_camera.is_backward_pressed {
                                    velocity += rotation * glam::Vec3::X;
                                }
                                if spherical_camera.is_left_pressed {
                                    velocity += rotation * glam::Vec3::Y;
                                }
                                if spherical_camera.is_right_pressed {
                                    velocity -= rotation * glam::Vec3::Y;
                                }
                                *lookat_target += velocity * (*speed);
                                *translation = spherical_camera.translation(*lookat_target);
                            }
                            _ => {}
                        }
                    }
                    _=>{}
                }
            }
        })],
    )
}
