use std::collections::HashSet;

use ambient_core::window::window_scale_factor;
use ambient_ecs::{
    components, generated::messages, world_events, Debuggable, Entity, FnSystem, Resource, System,
    SystemGroup, WorldEventsExt,
};
use glam::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use winit::event::{ModifiersState, TouchPhase};
pub use winit::event::{
    DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode,
    WindowEvent,
};
pub use ambient_ecs::generated::app::components::{
    cursor_position,last_touch_position
};
pub mod picking;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlayerRawInput {
    pub keys: HashSet<ambient_shared_types::VirtualKeyCode>,
    pub mouse_position: Vec2,
    pub mouse_delta: Vec2,
    pub mouse_wheel: f32,
    pub mouse_buttons: HashSet<ambient_shared_types::MouseButton>,
}
impl PlayerRawInput {
    pub fn clear(&mut self) {
        self.keys.clear();
        self.mouse_delta = vec2(0.0, 0.0);
        self.mouse_wheel = 0.0;
        self.mouse_buttons.clear();
    }
}

components!("input", {
    event_modifiers_change: ModifiersState,

    @[Debuggable, Resource]
    player_raw_input: PlayerRawInput,
    @[Debuggable, Resource]
    player_prev_raw_input: PlayerRawInput,
});

pub fn init_all_components() {
    picking::init_components();
    init_components();
}

pub fn event_systems() -> SystemGroup<Event<'static, ()>> {
    SystemGroup::new("inputs", vec![Box::new(InputSystem::new())])
}

pub fn cursor_lock_system(cursor_lock_rx: flume::Receiver<bool>) -> Box<dyn System + Send + Sync> {
    Box::new(FnSystem::new(move |world, _event| {
        for state in cursor_lock_rx.drain() {
            world
                .resource_mut(world_events())
                .add_message(messages::WindowCursorLockChange::new(state));
        }
    }))
}

pub fn resources() -> Entity {
    Entity::new()
        .with(player_raw_input(), Default::default())
        .with(player_prev_raw_input(), Default::default())
}
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalPosition
};
const DOUBLE_TAP_THRESHOLD: Duration = Duration::from_millis(300);
const TAP_MAX_DISTANCE: f64 = 200.0;

#[derive(Debug)]
struct TapTracker {
    last_tap_time: Option<Instant>,
    last_tap_position: Option<PhysicalPosition<f64>>,
    pub double_tab:bool
}

impl TapTracker {
    fn new() -> Self {
        Self {
            last_tap_time: None,
            last_tap_position: None,
            double_tab:false
        }
    }

    fn is_double_tap(&mut self, position: PhysicalPosition<f64>) -> bool {
        let now = Instant::now();
        if let Some(last_tap_time) = self.last_tap_time {
            if let Some(last_tap_position) = self.last_tap_position {
                if now.duration_since(last_tap_time) <= DOUBLE_TAP_THRESHOLD
                    && Self::distance(last_tap_position, position) <= TAP_MAX_DISTANCE
                {
                    self.last_tap_time = None; // Reset after double-tap detection
                    self.last_tap_position = None;
                    self.double_tab = true;
                    return true;
                }
            }
        }
        self.last_tap_time = Some(now);
        self.last_tap_position = Some(position);
        self.double_tab = false;
        false
    }

    fn distance(pos1: PhysicalPosition<f64>, pos2: PhysicalPosition<f64>) -> f64 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }
}
#[derive(Debug)]
pub struct InputSystem {
    modifiers: ModifiersState,
    is_focused: bool,
    tap_tracker: TapTracker
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            modifiers: ModifiersState::empty(),
            is_focused: false,
            tap_tracker: TapTracker{
                last_tap_time:None,
                last_tap_position:None,
                double_tab:false,
            }
        }
    }
}

impl System<Event<'static, ()>> for InputSystem {
    fn run(&mut self, world: &mut ambient_ecs::World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                &WindowEvent::Focused(focused) => {
                    self.is_focused = focused;
                    world
                        .resource_mut(world_events())
                        .add_message(messages::WindowFocusChange::new(focused));
                }
                WindowEvent::ReceivedCharacter(c) => {
                    // HACK: Drop the following characters as they will be produced
                    // by `KeyboardInput` instead.
                    if [
                        '\u{1b}', // Escape
                        '\t',     // Tab
                        '\u{7f}', // Delete
                        '\u{8}',  // Backspace
                        '\r',     // Return
                        '\n',     // Newline
                    ]
                    .contains(c)
                    {
                        return;
                    }

                    world
                        .resource_mut(world_events())
                        .add_message(messages::WindowKeyboardCharacter::new(c.to_string()));
                }

                WindowEvent::ModifiersChanged(mods) => {
                    self.modifiers = *mods;
                    world
                        .resource_mut(world_events())
                        .add_message(messages::WindowKeyboardModifiersChange::new(mods.bits()));
                }

                WindowEvent::CloseRequested => {
                    world
                        .resource_mut(world_events())
                        .add_message(messages::WindowClose::new());
                }

                WindowEvent::KeyboardInput { input, .. } => {
                    let keycode = input
                        .virtual_keycode
                        .map(|key| ambient_shared_types::VirtualKeyCode::from(key).to_string());

                    let modifiers = self.modifiers.bits();
                    let pressed = match input.state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    };

                    world.resource_mut(world_events()).add_message(
                        messages::WindowKeyboardInput::new(pressed, modifiers, keycode),
                    );
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    world.resource_mut(world_events()).add_message(
                        messages::WindowMouseInput::new(
                            match state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            },
                            ambient_shared_types::MouseButton::from(*button),
                        ),
                    );
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    world.resource_mut(world_events()).add_message(
                        messages::WindowMouseWheel::new(
                            match *delta {
                                MouseScrollDelta::LineDelta(x, y) => vec2(x, y),
                                MouseScrollDelta::PixelDelta(p) => vec2(p.x as f32, p.y as f32),
                            },
                            matches!(delta, MouseScrollDelta::PixelDelta(..)),
                        ),
                    );
                }
                WindowEvent::Touch ( touch ) => {
                    //tracing::info!("touch {:?}",touch);
                    let button = MouseButton::Left;
                    let p = vec2(touch.location.x as f32, touch.location.y as f32)
                    / world.get(world.resource_entity(),window_scale_factor()).unwrap() as f32;

                    match touch.phase{
                        TouchPhase::Started=>{

                            if !self.is_focused{
                                world.resource_mut(world_events()).add_message(
                                    messages::WindowFocusChange::new(true),
                                );
                                self.is_focused = true;
                            }
                            world
                            .set(world.resource_entity(), cursor_position(), p)
                            .unwrap();
                            //world.set(world.resource_entity(),last_touch_position(),p).unwrap();

                           if self.tap_tracker.is_double_tap(touch.location) {

                                world.resource_mut(world_events()).add_message(
                                    messages::WindowMouseInput::new(
                                        true,
                                        ambient_shared_types::MouseButton::from(button),
                                    ),
                                );
                           }

                        }
                        TouchPhase::Moved=>{
                            let scale_factor = world.get(world.resource_entity(),window_scale_factor()).unwrap();
                            let touch_after_scale_x= (touch.location.x/scale_factor) as f32;
                            let touch_after_scale_y= (touch.location.y/scale_factor) as f32;

                                let (c_x,c_y) ={
                                    let cp: &mut Vec2 = world
                                    .get_mut(world.resource_entity(), last_touch_position()).unwrap();
                                    let cp_c = cp.clone();
                                    cp.x = touch_after_scale_x;
                                    cp.y = touch_after_scale_y;
                                    if cp_c ==Vec2::ZERO{
                                        (touch_after_scale_x,touch_after_scale_y)
                                    }else{
                                        (cp_c.x,cp_c.y)
                                    }
                                };
                                let c_x = touch_after_scale_x-c_x;
                                let c_y = touch_after_scale_y-c_y;
                                    tracing::info!("c_x: {:?} c_x:{:?} ",c_x,c_y);
                                    world.resource_mut(world_events()).add_message(
                                        messages::WindowMouseMotion::new(vec2(
                                            c_x,
                                            c_y,
                                        )),
                                    );


                        }
                        _=>{
                            world.resource_mut(world_events()).add_message(
                                messages::WindowMouseInput::new(
                                    false,
                                    ambient_shared_types::MouseButton::from(button),
                                ),
                            );
                            world.resource_mut(world_events()).add_message(
                                messages::WindowMouseMotion::new(vec2(
                                    0.0,
                                    0.0,
                                )),
                            );
                            world.set(world.resource_entity(),last_touch_position(),vec2(0.0, 0.0)).unwrap();

                            // let cp = world
                            //     .get_mut(world.resource_entity(), last_touch_position()).unwrap();
                            // cp.x = 0.0;
                            // cp.y = 0.0;

                        }
                    }
                    if let Some(f) = touch.force{
                        match f{
                            winit::event::Force::Calibrated{
                                force, ..
                            }=>{
                                if force>=1.0{
                                    world.resource_mut(world_events()).add_message(
                                        messages::WindowMouseInput::new(
                                            true,
                                            ambient_shared_types::MouseButton::from(button),
                                        ),
                                    );
                                }
                            },
                            winit::event::Force::Normalized(force)=>{
                                if force>=1.0{
                                    world.resource_mut(world_events()).add_message(
                                        messages::WindowMouseInput::new(
                                            true,
                                            ambient_shared_types::MouseButton::from(button),
                                        ),
                                    );
                                }

                            }
                        }

                    }

                }
                _ => {
                    tracing::info!("other {:?}",event);
                }
            },

            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                world
                    .resource_mut(world_events())
                    .add_message(messages::WindowMouseMotion::new(vec2(
                        delta.0 as f32,
                        delta.1 as f32,
                    )));
            }
            _ => {
                //tracing::info!("other event {:?}",event);
            }
        }
    }
}

#[derive(Clone)]
pub struct MouseInput {
    pub state: ElementState,
    pub button: MouseButton,
}
