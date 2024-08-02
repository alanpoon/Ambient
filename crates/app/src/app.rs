use std::{future::Future, sync::Arc, time::Duration};

use ambient_cameras::assets_camera_systems;
pub use ambient_core::gpu;
use ambient_core::{
    asset_cache,
    async_ecs::async_ecs_systems,
    bounding::bounding_systems,
    camera::camera_systems,
    frame_index,
    hierarchy::dump_world_hierarchy_to_user,
    name, performance_samples, refcount_system, remove_at_time_system, runtime,
    transform::TransformSystem,
    window::{
        self, cursor_position, get_window_sizes, window_logical_size, window_physical_size, window_scale_factor, ExitStatus, WindowCtl
    },
    ClientTimeResourcesSystem, PerformanceSample, RuntimeKey,
};
use ambient_ecs::{
    components, generated::ui::components::focus, world_events, Debuggable, DynSystem, Entity,
    FrameEvent, MakeDefault, MaybeResource, System, SystemGroup, World, WorldEventsSystem,
};
use ambient_element::ambient_system;
use ambient_gizmos::{gizmos, Gizmos};
use ambient_gpu::{
    gpu::{Gpu, GpuKey},
    mesh_buffer::MeshBufferKey,
};
use ambient_gpu_ecs::{gpu_world, GpuWorld, GpuWorldSyncEvent, GpuWorldUpdate};
use ambient_input::last_touch_position;
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::{FpsCounter, FpsSample},
};
use ambient_procedurals::{procedural_storage, ProceduralStorage};
use ambient_renderer::lod::lod_system;
use ambient_settings::SettingsKey;
use ambient_sys::{task::RuntimeHandle, time::Instant};

use glam::{uvec2, vec2, IVec2, UVec2, Vec2};
use parking_lot::Mutex;
use crate::{renderers::{main_renderer, ui_renderer, MainRenderer, UiRenderer}, AppBuilder};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyEvent, WindowEvent::KeyboardInput, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{ModifiersState},
    window::{CursorGrabMode, Fullscreen, Window,WindowId},
};
use winit::keyboard::PhysicalKey;
use winit::keyboard::KeyCode;
use crate::{fps_stats,AsyncInit,window_title};
static mut QUIT:bool = false;
pub struct App {
    pub world: World,
    pub ctl_rx: flume::Receiver<WindowCtl>,
    pub systems: SystemGroup,
    pub gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub window_event_systems: SystemGroup<Event< ()>>,
    pub runtime: RuntimeHandle,
    //pub window: Option<Arc<Window>>,
    //event_loop: Option<EventLoop<()>>,
    pub fps: FpsCounter,
    #[cfg(feature = "profile")]
    _puffin: Option<puffin_http::Server>,
    pub modifiers: ModifiersState,

    pub window_focused: bool,
    pub update_title_with_fps_stats: bool,
    #[cfg(target_os = "unknown")]
    _drop_handles: Vec<Box<dyn std::fmt::Debug>>,
    pub current_time: Instant,

    #[cfg(target_os = "unknown")]
    force_resize_event_rx: flume::Receiver<(u32, u32)>,
}



impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("App");
        d.field("world", &self.world)
            .field("systems", &self.systems)
            .field("gpu_world_sync_systems", &self.gpu_world_sync_systems)
            .field("window_event_systems", &self.window_event_systems)
            .field("runtime", &self.runtime)
            //.field("window", &self.window)
            .field("fps", &self.fps)
            .field("window_focused", &self.window_focused);

        #[cfg(feature = "profile")]
        d.field("puffin", &true);
        #[cfg(not(feature = "profile"))]
        d.field("puffin", &false);

        d.finish()
    }
}
impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    #[cfg(target_os = "unknown")]
    pub fn spawn(mut self) {
        use winit::platform::web::EventLoopExtWebSys;

        let event_loop = self.event_loop.take().unwrap();

        tracing::debug!("Spawning event loop");
        event_loop.spawn(move |event, _, control_flow| {
            // HACK(philpax): hackfix for https://github.com/AmbientRun/Ambient/issues/923
            // remove after https://github.com/AmbientRun/Ambient/issues/1096
            // inject resize events if required
            if let Event::WindowEvent { window_id, .. } = &event {
                if let Ok((width, height)) = self.force_resize_event_rx.try_recv() {
                    self.handle_static_event(
                        &Event::WindowEvent {
                            window_id: *window_id,
                            event: WindowEvent::Resized(winit::dpi::PhysicalSize::new(
                                width, height,
                            )),
                        },
                        control_flow,
                    );
                }
            }

            // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
            // but https://github.com/rust-windowing/winit/issues/1968 restricts us
            if let Event::WindowEvent {
                window_id,
                event:
                    WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        scale_factor,
                    },
            } = &event
            {
                *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                self.handle_static_event(
                    &Event::WindowEvent {
                        window_id: *window_id,
                        event: WindowEvent::Resized(**new_inner_size),
                    },
                    control_flow,
                );
            } else if let Some(event) = event.to_static() {
                self.handle_static_event(&event, control_flow);
            } else {
                tracing::error!("Failed to convert event to static")
            }
        });
    }
    pub fn run_blocking(mut self,init: impl for<'x> AsyncInit<'x>) -> ExitStatus {
        ExitStatus::SUCCESS
    }
    // pub fn run_blocking(mut self,init: impl for<'x> AsyncInit<'x>) -> ExitStatus {
    //     if let Some(event_loop) = self.event_loop.take() {
    //         let init_c = Arc::new(Box::new(init));
    //         event_loop.run(move |event, _, control_flow| {
    //             // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
    //             // but https://github.com/rust-windowing/winit/issues/1968 restricts us
    //             if let Event::Resumed = event{
    //                 let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
    //                 let app_ = Arc::new(Mutex::new(None));
    //                 let app_c = app_.clone();
    //                 let init_c = init_c.clone();
    //                 rt.block_on(async move {
    //                     let mut app = AppBuilder::simple().build().await.unwrap();
    //                     *app_c.lock() = Some(app);

    //                 });
    //                 let app =app_.lock();
    //                 //init.call(app).await;
    //             }else{
    //                 if let Event::WindowEvent {
    //                     window_id,
    //                     event:
    //                         WindowEvent::ScaleFactorChanged {
    //                             new_inner_size,
    //                             scale_factor,
    //                         },
    //                 } = &event
    //                 {
    //                     *self.world.resource_mut(window_scale_factor()) = *scale_factor;
    //                     self.handle_static_event(
    //                         &Event::WindowEvent {
    //                             window_id: *window_id,
    //                             event: WindowEvent::Resized(**new_inner_size),
    //                         },
    //                         control_flow,
    //                     );
    //                 } else if let Some(event) = event.to_static() {
    //                     self.handle_static_event(&event, control_flow);
    //                 }
    //             }

    //         });
    //     } else {
    //         // Fake event loop in headless mode
    //         loop {
    //             let mut control_flow = ControlFlow::default();
    //             let exit_status =
    //                 self.handle_static_event(&Event::MainEventsCleared, &mut control_flow);
    //             if control_flow == ControlFlow::Exit {
    //                 return exit_status;
    //             }
    //         }
    //     }
    // }

    pub fn handle_static_event(
        &mut self,
        event: &Event<()>,
        control_flow: &mut ControlFlow,
    ) -> ExitStatus {
        *control_flow = ControlFlow::Poll;

        // From: https://github.com/gfx-rs/wgpu/issues/1783
        // TODO: According to the issue we should cap the framerate instead
        #[cfg(target_os = "macos")]
        if !self.window_focused {
            *control_flow = ControlFlow::Wait;
        }

        let world = &mut self.world;
        let systems = &mut self.systems;
        let gpu_world_sync_systems = &mut self.gpu_world_sync_systems;
        world.resource(gpu()).device.poll(wgpu::Maintain::Poll);

        self.window_event_systems.run(world, event);
        let quit =unsafe {
            QUIT
        };
        if quit{
            //*control_flow = ControlFlow::Exit;
            return ExitStatus::SUCCESS
        }
        match event {
            Event::WindowEvent{event:WindowEvent::RedrawRequested,..} => {
                let frame_start = Instant::now();
                let external_time = frame_start.duration_since(self.current_time);
                // Handle window control events
                for v in self.ctl_rx.try_iter() {
                    tracing::trace!(?v, "window control");
                    let gpu = world.resource(gpu()).clone();
                    let gpu = gpu.lock().unwrap();
                    match v {
                        WindowCtl::GrabCursor(mode) => {
                            if let Some(window) =gpu.get_view(){
                                match mode {
                                    CursorGrabMode::Confined | CursorGrabMode::Locked => {
                                        // Move the cursor to the centre of the window to ensure
                                        // the cursor is within the window and will not be locked
                                        // in place outside the window.
                                        //
                                        // Without this, on macOS, the cursor will be locked in place
                                        // and visible outside the window, which means the user can
                                        // click on other aspects of the operating system while
                                        // the cursor is locked.
                                        let (width, height) =
                                            <(u32, u32)>::from(window.inner_size());
                                        window
                                            .set_cursor_position(PhysicalPosition::new(
                                                width / 2,
                                                height / 2,
                                            ))
                                            .ok();

                                    }
                                    _ => {}
                                }
                                window.set_cursor_grab(mode).ok();
                            }
                        }
                        WindowCtl::ShowCursor(show) => {
                            if let Some(window) = gpu.get_view() {
                                window.set_cursor_visible(show);
                            }
                        }
                        WindowCtl::SetCursorIcon(icon) => {
                            if let Some(window) = gpu.get_view() {
                                window.set_cursor_icon(icon);
                            }
                        }
                        WindowCtl::SetTitle(title) => {
                            if let Some(window) = gpu.get_view() {
                                window.set_title(&title);
                            }
                        }
                        WindowCtl::SetFullscreen(fullscreen) => {
                            if let Some(window) =gpu.get_view() {
                                window.set_fullscreen(if fullscreen {
                                    Some(Fullscreen::Borderless(None))
                                } else {
                                    None
                                });
                            }
                        }
                        WindowCtl::ExitProcess(exit_status) => {
                            //*control_flow = ControlFlow::Exit;
                            return exit_status;
                        }
                    }
                }

                profiling::scope!("frame");
                world.next_frame();

                {
                    profiling::scope!("systems");
                    systems.run(world, &FrameEvent);
                    gpu_world_sync_systems.run(world, &GpuWorldSyncEvent);
                }
                let gpu = world.resource(gpu()).clone();
                if let Some(fps) = self.fps.frame_next() {
                    world
                        .set(world.resource_entity(), self::fps_stats(), fps.clone())
                        .unwrap();
                    if self.update_title_with_fps_stats {

                        if let Some(window) = gpu.get_view() {
                            window.set_title(&format!(
                                "{} [{}, {} entities]",
                                world.resource(window_title()),
                                fps.dump_both(),
                                world.len()
                            ));
                        }
                    }
                }

                if let Some(window) = gpu.get_view() {
                    window.request_redraw();
                }

                let frame_end = Instant::now();

                let frame_time = frame_end.duration_since(self.current_time);

                tracing::debug!(?external_time, ?frame_time, "frame time");
                self.current_time = frame_end;

                let samples = world.resource_mut(performance_samples());

                if samples.len() >= 128 {
                    samples.remove(0);
                }

                samples.push(PerformanceSample {
                    frame_time,
                    external_time,
                });

                profiling::finish_frame!();
            }

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Focused(focused) => {
                    self.window_focused = *focused;
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    tracing::info!("scale factor {:?}",*scale_factor);
                    *self.world.resource_mut(window_scale_factor()) = *scale_factor;
                }
                WindowEvent::Resized(size) => {
                    let gpu = world.resource(gpu()).clone();


                    //let size = uvec2(size.width, size.height);
                    let window = gpu.get_view().unwrap();
                    let scale_factor = window.scale_factor();
                    gpu.resize(*size);
                    let size = uvec2(size.width, size.height);
                    let logical_size = (size.as_dvec2() / scale_factor).as_uvec2();
                    tracing::info!("logical size {:?}",logical_size);
                    world
                        .set_if_changed(world.resource_entity(), window_physical_size(), size)
                        .unwrap();
                    world
                        .set_if_changed(
                            world.resource_entity(),
                            window_logical_size(),
                            logical_size,
                        )
                        .unwrap();

                }
                WindowEvent::CloseRequested => {
                    tracing::info!("Closing...");
                   // *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        if let PhysicalKey::Code(KeyCode::KeyQ) = event.physical_key {
                            // if self.modifiers.logo() {
                            //    // *control_flow = ControlFlow::Exit;
                            // }
                        }
                    }
                    // if let Some(keycode) = input.virtual_keycode {
                    //     if input.state == ElementState::Pressed {
                    //         if let VirtualKeyCode::Q = keycode {
                    //             if self.modifiers.logo() {
                    //                 *control_flow = ControlFlow::Exit;
                    //             }
                    //         }
                    //     }
                    // }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.modifiers = state.state();
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.window_focused {
                        let gpu = world.resource(gpu()).clone();
                        let p = vec2(position.x as f32, position.y as f32)
                            / gpu.get_view()
                                .map(|x| x.scale_factor() as f32)
                                .unwrap_or(1.);
                        world
                            .set(world.resource_entity(), cursor_position(), p)
                            .unwrap();
                    }
                }
                // WindowEvent::Destroyed => {
                //     *control_flow = ControlFlow::;
                // }
                _ => {}
            },
            _ => {}
        }
        ExitStatus::SUCCESS
    }
    pub fn add_system(&mut self, system: DynSystem) -> &mut Self {
        self.systems.add(system);
        self
    }
}
