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
    gpu::{Gpu, GpuKey, WrapWindow,get_scale_factor,get_view_size},
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
use renderers::{main_renderer, ui_renderer, MainRenderer, UiRenderer};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, ModifiersState, TouchPhase, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Fullscreen, Window, WindowBuilder},
};
use objc::{runtime::Object, *};
use libc::c_void;
pub mod ffi;
static mut QUIT:bool = false;
static mut LOADED:bool = false;
mod renderers;
fn default_title() -> String {
    "ambient".into()
}

components!("app", {
    @[MakeDefault[default_title], Debuggable, MaybeResource]
    window_title: String,
    fps_stats: FpsSample,

});

pub fn init_all_components() {
    ambient_ecs::init_components();
    ambient_core::init_all_components();
    ambient_element::init_components();
    ambient_animation::init_all_components();
    ambient_gizmos::init_components();
    ambient_cameras::init_all_components();
    init_components();
    ambient_renderer::init_all_components();
    ambient_ui_native::init_all_components();
    ambient_input::init_all_components();
    ambient_model::init_components();
    ambient_cameras::init_all_components();
    renderers::init_components();
    ambient_procedurals::init_components();
}

pub fn gpu_world_sync_systems(gpu: Arc<Gpu>) -> SystemGroup<GpuWorldSyncEvent> {
    SystemGroup::new(
        "gpu_world",
        vec![
            // Note: All Gpu sync systems must run immediately after GpuWorldUpdate, as that's the only time we know
            // the layout of the GpuWorld is correct
            Box::new(GpuWorldUpdate(gpu.clone())),
            Box::new(ambient_core::transform::transform_gpu_systems(gpu.clone())),
            Box::new(ambient_renderer::gpu_world_systems(gpu.clone())),
            Box::new(ambient_core::bounding::gpu_world_systems(gpu.clone())),
            Box::new(ambient_ui_native::layout::gpu_world_systems(gpu.clone())),
        ],
    )
}

pub fn world_instance_systems(full: bool) -> SystemGroup {
    SystemGroup::new(
        "world_instance",
        vec![
            Box::new(ClientTimeResourcesSystem::new()),
            Box::new(async_ecs_systems()),
            remove_at_time_system(),
            refcount_system(),
            Box::new(ambient_core::hierarchy::systems()),
            Box::new(WorldEventsSystem),
            Box::new(ambient_focus::systems()),
            if full {
                Box::new(ambient_input::picking::frame_systems())
            } else {
                Box::new(DummySystem)
            },
            Box::new(lod_system()),
            Box::new(ambient_renderer::systems()),
            Box::new(ambient_system()),
            if full {
                Box::new(ambient_ui_native::systems())
            } else {
                Box::new(DummySystem)
            },
            Box::new(ambient_model::model_systems()),
            Box::new(ambient_animation::animation_systems()),
            Box::new(TransformSystem::new()),
            Box::new(ambient_renderer::skinning::skinning_systems()),
            Box::new(bounding_systems()),
            Box::new(camera_systems()),
            Box::new(ambient_procedurals::client_systems()),
        ],
    )
}

pub struct AppResources {
    pub assets: AssetCache,
    pub gpu: Arc<Gpu>,
    pub runtime: RuntimeHandle,
    pub ctl_tx: flume::Sender<WindowCtl>,
    window_physical_size: UVec2,
    window_logical_size: UVec2,
    window_scale_factor: f64,
}

impl AppResources {
    pub fn from_world(world: &World) -> Self {
        Self {
            assets: world.resource(self::asset_cache()).clone(),
            gpu: world.resource(self::gpu()).clone(),
            runtime: world.resource(self::runtime()).clone(),
            ctl_tx: world.resource(ambient_core::window::window_ctl()).clone(),
            window_physical_size: *world.resource(ambient_core::window::window_physical_size()),
            window_logical_size: *world.resource(ambient_core::window::window_logical_size()),
            window_scale_factor: *world.resource(ambient_core::window::window_scale_factor()),
        }
    }
}

pub fn world_instance_resources(resources: AppResources) -> Entity {
    Entity::new()
        .with(name(), "Resources".to_string())
        .with(self::gpu(), resources.gpu.clone())
        .with(gizmos(), Gizmos::new())
        .with(self::runtime(), resources.runtime)
        .with(self::window_title(), "".to_string())
        .with(self::fps_stats(), FpsSample::default())
        .with(self::performance_samples(), Vec::new())
        .with(self::asset_cache(), resources.assets.clone())
        .with(world_events(), Default::default())
        .with(frame_index(), 0_usize)
        .with(ambient_core::window::cursor_position(), Vec2::ZERO)
        .with(ambient_core::window::last_touch_position(), Vec2::ZERO)
        //.with(ambient_core::window::window_dpi(), 1)
        .with(
            gpu_world(),
            GpuWorld::new_arced(&resources.gpu, resources.assets),
        )
        .with_merge(ambient_core::time_resources_start(Duration::ZERO))
        .with_merge(ambient_input::resources())
        .with_merge(ambient_input::picking::resources())
        .with_merge(ambient_core::async_ecs::async_ecs_resources())
        .with(
            ambient_core::window::window_physical_size(),
            resources.window_physical_size,
        )
        .with(
            ambient_core::window::window_logical_size(),
            resources.window_logical_size,
        )
        .with(
            ambient_core::window::window_scale_factor(),
            resources.window_scale_factor,
        )
        .with(ambient_core::window::window_ctl(), resources.ctl_tx)
        .with(procedural_storage(), ProceduralStorage::new())
        .with(focus(), Default::default())
}

pub struct AppBuilder {
    pub event_loop: Option<EventLoop<()>>,
    pub asset_cache: Option<AssetCache>,
    pub ui_renderer: bool,
    pub main_renderer: bool,
    pub examples_systems: bool,
    pub headless: Option<UVec2>,
    pub update_title_with_fps_stats: bool,
    ctl: Option<(flume::Sender<WindowCtl>, flume::Receiver<WindowCtl>)>,
    pub window_position_override: Option<IVec2>,
    pub window_size_override: Option<UVec2>,
    #[cfg(target_os = "unknown")]
    pub parent_element: Option<web_sys::HtmlElement>,

}

pub trait AsyncInit<'a> {
    type Future: 'a + Future<Output = ()>;
    fn call(self, app: &'a mut App) -> Self::Future;
}

impl<'a, F, Fut> AsyncInit<'a> for F
where
    Fut: 'a + Future<Output = ()>,
    F: FnOnce(&'a mut App) -> Fut,
{
    type Future = Fut;

    fn call(self, app: &'a mut App) -> Self::Future {
        (self)(app)
    }
}
#[cfg(target_os="android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os="android")]
pub trait AsyncInitAndroid<'a> {
    type Future: 'a + Future<Output = ()>;
    fn call(self, app: &'a mut App,android_app:AndroidApp) -> Self::Future;
}
#[cfg(target_os="android")]

impl<'a, F, Fut> AsyncInitAndroid<'a> for F
where
    Fut: 'a + Future<Output = ()>,
    F: FnOnce(&'a mut App,AndroidApp) -> Fut,
{
    type Future = Fut;

    fn call(self, app: &'a mut App,android_app: AndroidApp) -> Self::Future {
        (self)(app,android_app)
    }
}
impl AppBuilder {
    pub fn new() -> Self {
        Self {
            event_loop: None,
            asset_cache: None,
            ui_renderer: false,
            main_renderer: true,
            examples_systems: false,
            headless: None,
            update_title_with_fps_stats: true,
            ctl: None,
            window_position_override: None,
            window_size_override: None,
            #[cfg(target_os = "unknown")]
            parent_element: None,
        }
    }
    pub fn simple() -> Self {
        Self::new().examples_systems(true)
    }
    pub fn simple_ui() -> Self {
        Self::new()
            .ui_renderer(true)
            .main_renderer(false)
            .examples_systems(true)
    }
    pub fn simple_dual() -> Self {
        Self::new().ui_renderer(true).main_renderer(true)
    }
    pub fn with_event_loop(mut self, event_loop: EventLoop<()>) -> Self {
        self.event_loop = Some(event_loop);
        self
    }

    pub fn with_asset_cache(mut self, asset_cache: AssetCache) -> Self {
        self.asset_cache = Some(asset_cache);
        self
    }

    pub fn ui_renderer(mut self, value: bool) -> Self {
        self.ui_renderer = value;
        self
    }

    pub fn main_renderer(mut self, value: bool) -> Self {
        self.main_renderer = value;
        self
    }

    pub fn examples_systems(mut self, value: bool) -> Self {
        self.examples_systems = value;
        self
    }

    pub fn headless(mut self, value: Option<UVec2>) -> Self {
        self.headless = value;
        self
    }

    pub fn update_title_with_fps_stats(mut self, value: bool) -> Self {
        self.update_title_with_fps_stats = value;
        self
    }

    pub fn with_window_position_override(mut self, position: IVec2) -> Self {
        self.window_position_override = Some(position);
        self
    }

    pub fn with_window_size_override(mut self, size: UVec2) -> Self {
        self.window_size_override = Some(size);
        self
    }

    #[cfg(target_os = "unknown")]
    pub fn parent_element(mut self, value: Option<web_sys::HtmlElement>) -> Self {
        self.parent_element = value;
        self
    }

    pub fn window_ctl(
        mut self,
        ctl_tx: flume::Sender<WindowCtl>,
        ctl_rx: flume::Receiver<WindowCtl>,
    ) -> Self {
        self.ctl = Some((ctl_tx, ctl_rx));

        self
    }

    pub async fn build(self,window:Option<Arc<Window>>) -> anyhow::Result<App> {
        crate::init_all_components();

        let runtime: RuntimeHandle = RuntimeHandle::current();

        let assets = self
            .asset_cache
            .unwrap_or_else(|| AssetCache::new(runtime.clone()));

        let settings = SettingsKey.get(&assets);

        // let (window, event_loop) = if self.headless.is_some() {
        //     (None, None)
        // } else {
        //     //let event_loop = self.event_loop.unwrap_or_default();
        //     let window = WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
        //         width: settings.render.resolution().0,
        //         height: settings.render.resolution().1,
        //     });
        //     let window = if let Some(position) = self.window_position_override {
        //         window.with_position(winit::dpi::LogicalPosition {
        //             x: position.x,
        //             y: position.y,
        //         })
        //     } else {
        //         window
        //     };
        //     let window = if let Some(size) = self.window_size_override {
        //         window.with_inner_size(winit::dpi::LogicalSize {
        //             width: size.x,
        //             height: size.y,
        //         })
        //     } else {
        //         window
        //     };
        //     let window = Arc::new(window.build(&event_loop).unwrap());
        //     (Some(window), Some(event_loop))
        // };

        let (cursor_lock_tx, cursor_lock_rx) = flume::unbounded::<bool>();

        // This isn't necessary on native
        #[cfg(not(target_os = "unknown"))]
        let _ = cursor_lock_tx;

        #[cfg(target_os = "unknown")]
        let mut drop_handles: Vec<Box<dyn std::fmt::Debug>> = Vec::new();

        #[cfg(target_os = "unknown")]
        // Insert a canvas element for the window to attach to
        let force_resize_event_rx = {
            use winit::platform::web::WindowExtWebSys;

            let window = window.as_ref().expect("this should not be possible");
            let canvas = window.canvas();
            let document = web_sys::window().unwrap().document().unwrap();

            let target = self
                .parent_element
                .unwrap_or_else(|| document.body().unwrap());

            use wasm_bindgen::prelude::*;

            let on_context_menu = Closure::<dyn Fn(_)>::new(|event: web_sys::MouseEvent| {
                event.prevent_default();
            });

            canvas.set_oncontextmenu(Some(on_context_menu.as_ref().unchecked_ref()));
            drop_handles.push(Box::new(on_context_menu));

            // HACK: Listen for pointer lock change here to ensure that the guest
            // is notified that the pointer is no longer locked, so that they can
            // update their state accordingly.
            //
            // It would be nice to move this into the input systems, but I don't
            // want to leak too much information about the running environment
            // into other code.
            let on_pointer_lock_change = Closure::<dyn Fn()>::new({
                let canvas = canvas.clone();
                let document = document.clone();
                move || {
                    let is_locked =
                        document.pointer_lock_element().as_ref() == Some(canvas.as_ref());

                    let _ = cursor_lock_tx.send(is_locked);
                }
            });
            document
                .add_event_listener_with_callback_and_bool(
                    "pointerlockchange",
                    on_pointer_lock_change.as_ref().unchecked_ref(),
                    false,
                )
                .unwrap();
            drop_handles.push(Box::new(on_pointer_lock_change));

            // Updates the canvas to match the target size. Returns the target size/max size, and the real size.
            fn update_canvas_size(
                target: &web_sys::Element,
                canvas: &web_sys::HtmlCanvasElement,
            ) -> ((i32, i32), (u32, u32)) {
                // Get the screen's available width and height
                let window = web_sys::window().unwrap();

                let max_width = target.client_width();
                let max_height = target.client_height();

                // Get device pixel ratio
                let device_pixel_ratio = window.device_pixel_ratio();

                // Calculate the real dimensions of the canvas considering the device pixel ratio
                let real_width = (max_width as f64 * device_pixel_ratio) as u32;
                let real_height = (max_height as f64 * device_pixel_ratio) as u32;

                // Set the canvas dimensions using the real dimensions
                canvas.set_width(real_width);
                canvas.set_height(real_height);

                // Set a background color for the canvas to make it easier to tell where the canvas is for debugging purposes.
                // Use the maximum available width and height as the canvas dimensions.
                canvas.style().set_css_text(&format!(
                    "background-color: black; width: {}px; height: {}px; z-index: 50",
                    max_width, max_height
                ));

                ((max_width, max_height), (real_width, real_height))
            }

            let ((max_width, max_height), _) = update_canvas_size(&target, &canvas);

            // HACK(philpax): hackfix for https://github.com/AmbientRun/Ambient/issues/923
            // remove after https://github.com/AmbientRun/Ambient/issues/1096
            let (force_resize_event_tx, force_resize_event_rx) = flume::unbounded();
            let resize_callback = Closure::<dyn Fn()>::new({
                let canvas = canvas.clone();
                let target = target.clone();
                move || {
                    let (_, real_size) = update_canvas_size(&target, &canvas);
                    let _ = force_resize_event_tx.send(real_size);
                }
            });
            let resize_observer =
                web_sys::ResizeObserver::new(resize_callback.as_ref().unchecked_ref()).unwrap();
            resize_observer.observe(&target);
            drop_handles.push(Box::new(resize_callback));
            drop_handles.push(Box::new(resize_observer));

            target.append_child(&canvas).unwrap();
            force_resize_event_rx
        };

        #[cfg(feature = "profile")]
        let puffin_server = {
            let puffin_addr = format!(
                "0.0.0.0:{}",
                std::env::var("PUFFIN_PORT")
                    .ok()
                    .and_then(|port| port.parse::<u16>().ok())
                    .unwrap_or(puffin_http::DEFAULT_PORT)
            );
            match puffin_http::Server::new(&puffin_addr) {
                Ok(server) => {
                    tracing::debug!("Puffin server running on {}", puffin_addr);
                    puffin::set_scopes_on(true);
                    Some(server)
                }
                Err(err) => {
                    tracing::error!("Failed to start puffin server: {:?}", err);
                    None
                }
            }
        };

        #[cfg(not(target_os = "unknown"))]
        let _ = thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Max);

        let mut world = World::new("main_app", ambient_ecs::WorldContext::App);
        let gpu = Arc::new(Gpu::with_config(window.as_deref(), true, &settings.render).await?);
        tracing::info!("settings {:?}",settings);

        tracing::debug!("Inserting runtime");
        RuntimeKey.insert(&assets, runtime.clone());
        GpuKey.insert(&assets, gpu.clone());
        // WindowKey.insert(&assets, window.clone());

        tracing::debug!("Inserting app resources");
        let (ctl_tx, ctl_rx) = self.ctl.unwrap_or_else(flume::unbounded);

        let (window_physical_size, window_logical_size, window_scale_factor) =
          {
            let headless_size = self.headless.unwrap();
            (headless_size, headless_size, 1.)
          };
        tracing::info!("window_physical_size {:?},window_logical_size {:?}.{:?}",window_physical_size,window_logical_size,window_scale_factor);
        let app_resources = AppResources {
            gpu: gpu.clone(),
            runtime: runtime.clone(),
            assets,
            ctl_tx,
            window_physical_size,
            window_logical_size,
            window_scale_factor,
        };

        let resources = world_instance_resources(app_resources);

        world
            .add_components(world.resource_entity(), resources)
            .unwrap();
        tracing::debug!("Setup renderers");
        if self.ui_renderer || self.main_renderer {
            // let _span = info_span!("setup_renderers").entered();
            if !self.main_renderer {
                tracing::debug!("Setting up UI renderer");
                let renderer = Arc::new(Mutex::new(UiRenderer::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                tracing::debug!("Setting up Main renderer");
                let renderer =
                    MainRenderer::new(&gpu, &mut world, self.ui_renderer, self.main_renderer);
                tracing::debug!("Created main renderer");
                let renderer = Arc::new(Mutex::new(renderer));
                world.add_resource(main_renderer(), renderer);
            }
        }

        tracing::debug!("Adding window event systems");

        let mut window_event_systems = SystemGroup::new(
            "window_event_systems",
            vec![
                Box::new(assets_camera_systems()),
                Box::new(ambient_input::event_systems()),
                Box::new(renderers::systems()),
            ],
        );
        if self.examples_systems {
            window_event_systems.add(Box::new(ExamplesSystem));
        }

        Ok(App {
            window_focused: true,
            window,
            runtime,
            systems: SystemGroup::new(
                "app",
                vec![
                    Box::new(MeshBufferUpdate),
                    Box::new(world_instance_systems(true)),
                    ambient_input::cursor_lock_system(cursor_lock_rx),
                ],
            ),
            world,
            gpu_world_sync_systems: gpu_world_sync_systems(gpu.clone()),
            window_event_systems,
            //event_loop,

            fps: FpsCounter::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
            modifiers: Default::default(),
            ctl_rx,
            current_time: Instant::now(),
            update_title_with_fps_stats: self.update_title_with_fps_stats,
            #[cfg(target_os = "unknown")]
            _drop_handles: drop_handles,

            #[cfg(target_os = "unknown")]
            force_resize_event_rx,

            //view:None
        })
    }
    #[cfg(target_os="ios")]
    pub async fn build_view(self,view:Option<*mut Object>, metal_layer:Option<*mut c_void> ) -> anyhow::Result<App>{
        crate::init_all_components();

        let runtime: RuntimeHandle = RuntimeHandle::current();

        let assets = self
            .asset_cache
            .unwrap_or_else(|| AssetCache::new(runtime.clone()));

        let settings = SettingsKey.get(&assets);
        let (cursor_lock_tx, cursor_lock_rx) = flume::unbounded::<bool>();
        let mut world = World::new("main_app", ambient_ecs::WorldContext::App);
        let gpu = Arc::new(Gpu::with_view(view,metal_layer, true, &settings.render).await?);
        tracing::info!("settings {:?}",settings);

        tracing::debug!("Inserting runtime");
        RuntimeKey.insert(&assets, runtime.clone());
        GpuKey.insert(&assets, gpu.clone());
        // WindowKey.insert(&assets, window.clone());

        tracing::debug!("Inserting app resources");
        let (ctl_tx, ctl_rx) = self.ctl.unwrap_or_else(flume::unbounded);

        // let (window_physical_size, window_logical_size, window_scale_factor) =
        //     if let Some(window) = window.as_ref() {
        //         get_window_sizes(window)
        //     } else {
        //         let headless_size = self.headless.unwrap();
        //         (headless_size, headless_size, 1.)
        //     };
        let (window_physical_size, window_logical_size, window_scale_factor) ={
            (UVec2::new(500,500),UVec2::new(500, 500),1.0)
        };
        tracing::info!("window_physical_size {:?},window_logical_size {:?}.{:?}",window_physical_size,window_logical_size,window_scale_factor);
        let app_resources = AppResources {
            gpu: gpu.clone(),
            runtime: runtime.clone(),
            assets,
            ctl_tx,
            window_physical_size,
            window_logical_size,
            window_scale_factor,
        };

        let resources = world_instance_resources(app_resources);

        world
            .add_components(world.resource_entity(), resources)
            .unwrap();
        tracing::debug!("Setup renderers");
        if self.ui_renderer || self.main_renderer {
            // let _span = info_span!("setup_renderers").entered();
            if !self.main_renderer {
                tracing::debug!("Setting up UI renderer");
                let renderer = Arc::new(Mutex::new(UiRenderer::new(&mut world)));
                world.add_resource(ui_renderer(), renderer);
            } else {
                tracing::debug!("Setting up Main renderer");
                let renderer =
                    MainRenderer::new(&gpu, &mut world, self.ui_renderer, self.main_renderer);
                tracing::debug!("Created main renderer");
                let renderer = Arc::new(Mutex::new(renderer));
                world.add_resource(main_renderer(), renderer);
            }
        }

        tracing::debug!("Adding window event systems");

        let mut window_event_systems = SystemGroup::new(
            "window_event_systems",
            vec![
                Box::new(assets_camera_systems()),
                Box::new(ambient_input::event_systems()),
                Box::new(renderers::systems()),
            ],
        );
        if self.examples_systems {
            window_event_systems.add(Box::new(ExamplesSystem));
        }
        let window = None;
        Ok(App {
            window_focused: true,
            window,
            runtime,
            systems: SystemGroup::new(
                "app",
                vec![
                    Box::new(MeshBufferUpdate),
                    Box::new(world_instance_systems(true)),
                    ambient_input::cursor_lock_system(cursor_lock_rx),
                ],
            ),
            world,
            gpu_world_sync_systems: gpu_world_sync_systems(gpu.clone()),
            window_event_systems,
            //event_loop,

            fps: FpsCounter::new(),
            #[cfg(feature = "profile")]
            _puffin: puffin_server,
            modifiers: Default::default(),
            ctl_rx,
            current_time: Instant::now(),
            update_title_with_fps_stats: self.update_title_with_fps_stats,
            #[cfg(target_os = "unknown")]
            _drop_handles: drop_handles,

            #[cfg(target_os = "unknown")]
            force_resize_event_rx,
            //view:None
        })
    }

    // Runs the app by blocking the main thread
    // #[cfg(not(target_os = "unknown"))]
    // pub fn block_on(self, init: impl for<'x> AsyncInit<'x>) {
    //     let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();

    //     rt.block_on(async move {
    //         let mut app = self.build().await.unwrap();

    //         init.call(&mut app).await;

    //         app.run_blocking();
    //     });
    // }

    // Finalizes the app and enters the main loop
    // pub async fn run(self, init: impl FnOnce(&mut App, RuntimeHandle)) -> ExitStatus {
    //     let mut app = self.build().await.unwrap();
    //     let runtime = app.runtime.clone();
    //     init(&mut app, runtime);
    //     app.run_blocking()
    // }

    // #[inline]
    // pub async fn run_world(self, init: impl FnOnce(&mut World)) -> ExitStatus {
    //     self.run(|app, _| init(&mut app.world)).await
    // }
}
/// Creates a 2-dimensional vector.
// #[inline(always)]
// pub const fn uvec2(x: u32, y: u32) -> UVec2 {
//     UVec2::new(x, y)
// }
pub struct AppWrapper{
    pub app : Arc<Mutex<Option<App>>>,
    pub event_loop: Option<EventLoop<()>>,
    pub window: Option<Arc<Window>>,
    pub once:bool,
    pub runtime:Arc<tokio::runtime::Runtime>,
}
#[cfg(target_os = "android")]
extern crate jni;
#[cfg(target_os = "android")]
use jni::objects::JClass;
#[cfg(target_os = "android")]
use jni::sys::jstring;
#[cfg(target_os = "android")]
use jni::JNIEnv;
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_dev_rustropy_wry2_SecondActivity_closeActivity(env: JNIEnv, _class: JClass){
    tracing::info!("closing quit = true");
   unsafe{
    QUIT = true;
   }
}

impl AppWrapper{
    pub fn new()->AppWrapper{
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new();
        // .with_inner_size(winit::dpi::LogicalSize {
        //     width: 1200,
        //     height: 800,
        // });
        let window = window.build(&event_loop).unwrap();
        let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();

        AppWrapper{
            app:Arc::new(Mutex::new(None)),
            event_loop:Some(event_loop),
            window:Some(Arc::new(window)),
            once:false,
            runtime:Arc::new(rt)
        }
    }
    #[cfg(target_os="ios")]
    pub fn new_with_view(ios_obj: ffi::IOSViewObj,box_c:Box<dyn Fn()>)->AppWrapper{
        let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
        let runtime = rt.handle();
        let assets: AssetCache = AssetCache::new(runtime.clone());
        let _settings = SettingsKey.get(&assets);
        box_c();
        let scale_factor = get_scale_factor(ios_obj.view);
        println!("scale_factor {:?}",scale_factor);
        let (width,height) = get_view_size(ios_obj.view,scale_factor);
        let headless = Some(uvec2(width, height));
        println!("headless {:?}",headless);
        let mut app =  rt.block_on(async move {
            AppBuilder::new()
               .ui_renderer(true)
               .with_asset_cache(assets)
               .headless(headless)
               .update_title_with_fps_stats(false)
               .build_view(Some(ios_obj.view),Some(ios_obj.metal_layer)).await.unwrap()
       });
       let mut app_c = app_.clone();
       std::thread::spawn(move||{
        //self.runtime.block_on(async move{
           rt.block_on(async move{
            if let Some(app ) =app_c.lock().as_mut(){
                i_c.call(app).await;
            }
            use std::time::{Duration};
            use std::thread::sleep;
            loop{
                sleep(Duration::new(5,0));
            }
        });
        });
       *app.world.resource_mut(window_scale_factor()) = scale_factor as f64;
       //*app_.lock() = Some(app);

        AppWrapper{
            app:Arc::new(Mutex::new(Some(app))),
            event_loop:None,
            window:None,
            once:false,
            runtime:Arc::new(rt)
        }
    }
    pub fn new_with_event_loop(event_loop:EventLoop<()>)->AppWrapper{
        // let window = WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
        //     width: 1200,
        //     height: 800,
        // });
        let window = WindowBuilder::new();
        let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
        let window = window.build(&event_loop).unwrap();

        AppWrapper{
            app:Arc::new(Mutex::new(None)),
            event_loop:Some(event_loop),
            window:Some(Arc::new(window)),
            once:false,
            runtime:Arc::new(rt)
        }
    }
    #[cfg(not(target_os="android"))]
    pub fn run_blocking(mut self,init: impl for<'x> AsyncInit<'x>  +Copy+ Clone+Send+'static,box_c:Box<dyn Fn()>) {
        use std::ops::DerefMut;

        use tracing::event;

        if let Some(event_loop) = self.event_loop.take() {
            let mut quit= false;
            event_loop.run(move |event, _, control_flow| {
                // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
                // but https://github.com/rust-windowing/winit/issues/1968 restricts us
                if let Event::Resumed = event{
                    if self.once{
                        return
                    }
                    let window = self.window.clone();
                    let app_ = self.app.clone();
                    let i_c = init.clone();
                    let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
                    let runtime = rt.handle();
                    let assets: AssetCache = AssetCache::new(runtime.clone());
                    let _settings = SettingsKey.get(&assets);
                    // use ambient_physics::physx::PhysicsKey;
                    // PhysicsKey.get(&assets); // Load physics

                    box_c();
                    let in_size: winit::dpi::PhysicalSize<u32> = self.window.clone().unwrap().inner_size();
                    let width = in_size.width;
                    let height = in_size.height;
                    let headless = Some(uvec2(width, height));
                    let mut app =  rt.block_on(async move {
                        AppBuilder::new()
                           .ui_renderer(true)
                           .with_asset_cache(assets)
                           .headless(headless)
                           .update_title_with_fps_stats(false)
                           .build(window).await.unwrap()
                    });
                       // thread code
                    let scale_factor = self
                       .window
                       .as_ref()
                       .map(|x| x.scale_factor() as f32)
                       .unwrap_or(1.) as f64;
                   *app.world.resource_mut(window_scale_factor()) = scale_factor;
                   *app_.lock() = Some(app);
                   unsafe{
                       LOADED = true;
                   }
                   //use tokio::time::{sleep, Duration};
                   let quit = unsafe{
                       QUIT
                   };
                   let mut app_c = app_.clone();
                   std::thread::spawn(move||{
                       rt.block_on(async move{
                          if let Some(app ) =app_c.lock().as_mut(){
                            i_c.call(app).await;
                           }
                           use std::time::{Duration};
                           use std::thread::sleep;
                           loop{
                               sleep(Duration::new(5,0));

                           }
                       });
                   });
                    self.once = true;
                }else{
                    let app_ = self.app.clone();
                    let mut app_ = app_.lock();
                    if let Some(ref mut app) = *app_{
                        if let Event::WindowEvent {
                            window_id,
                            event:
                                WindowEvent::ScaleFactorChanged {
                                    new_inner_size,
                                    scale_factor,
                                },
                        } = &event
                        {

                            *app.world.resource_mut(window_scale_factor()) = *scale_factor;
                            app.handle_static_event(
                                &Event::WindowEvent {
                                    window_id: *window_id,
                                    event: WindowEvent::Resized(**new_inner_size),
                                },
                                control_flow,
                            );
                        } else if let Some(event_) = event.to_static() {
                            if let Event::WindowEvent {window_id,  event } = event_.clone(){
                                match event{
                                    WindowEvent::Destroyed =>{
                                        quit = true;
                                        *control_flow = ControlFlow::Exit;
                                    }
                                    _=>{}
                                }
                            }
                            app.handle_static_event(&event_, control_flow);
                        }
                    }

                }

            });
        } else {
            // Fake event loop in headless mode
            loop {
                let mut control_flow = ControlFlow::default();
                //let exit_status =
                    //self.handle_static_event(&Event::MainEventsCleared, &mut control_flow);
                if control_flow == ControlFlow::Exit {
                    //return exit_status;
                }
            }
        }
    }
    #[cfg(target_os="android")]
    pub fn run_blocking(mut self,init: impl for<'x> AsyncInitAndroid<'x>  +Copy+ Clone+Send+'static,android_app:AndroidApp,box_c:Box<dyn Fn()>) {
        if let Some(event_loop) = self.event_loop.take() {
            event_loop.run(move |event: Event<()>, _, control_flow| {
                // HACK(philpax): treat dpi changes as resize events. Ideally we'd handle this in handle_event proper,
                // but https://github.com/rust-windowing/winit/issues/1968 restricts us
                *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() +  std::time::Duration::from_millis(16));
                tracing::info!("ControlFlow::Wait");
                // unsafe{
                //     if LOADED{
                //         tracing::info!("ControlFlow::Wait");
                //         *control_flow = ControlFlow::WaitUntil(());
                //     }
                // }
                if let Event::Resumed = event {
                    if self.once{

                        return
                    }
                    //*control_flow = ControlFlow::Wait;
                    let window = self.window.clone();
                    let app_ = self.app.clone();
                    let android_app_c = android_app.clone();
                    let i_c = init.clone();
                    let mut rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();

                    let runtime = rt.handle();
                    let assets: AssetCache = AssetCache::new(runtime.clone());
                    let _settings = SettingsKey.get(&assets);
                    box_c();
                    let in_size: winit::dpi::PhysicalSize<u32> = self.window.clone().unwrap().inner_size();
                    let width = in_size.width;
                    let height = in_size.height;
                    let headless = Some(uvec2(width, height));
                    let scale_factor = self
                    .window
                    .as_ref()
                    .map(|x| x.scale_factor() as f32)
                    .unwrap_or(1.) as f64;
                    tracing::info!("scale_factor {:?}",scale_factor);
                    let mut app =  rt.block_on(async move {
                         AppBuilder::new()
                            .ui_renderer(true)
                            .with_asset_cache(assets)
                            .headless(headless)
                            .update_title_with_fps_stats(false)
                            .build(window).await.unwrap()
                    });
                        // thread code

                    *app.world.resource_mut(window_scale_factor()) = scale_factor;
                    *app_.lock() = Some(app);

                    unsafe{
                        LOADED = true;
                    }
                    //use tokio::time::{sleep, Duration};
                    let quit = unsafe{
                        QUIT
                    };
                    let mut app_c = app_.clone();
                    std::thread::spawn(move||{
                        rt.block_on(async move{
                            if let Some(app ) =app_c.lock().as_mut(){
                                i_c.call(app,android_app_c).await;
                                use std::time::{Duration};
                                use std::thread::sleep;
                                loop{
                                    sleep(Duration::new(5,0));

                                }
                              }

                        });
                    });

                    self.once = true;
                }
                else{
                    if let Event::WindowEvent {
                        window_id,
                        event:
                            WindowEvent::ScaleFactorChanged {
                                new_inner_size,
                                scale_factor,
                            },
                    } = &event
                    {
                        let app_ = self.app.clone();
                        let mut app_ = app_.lock();
                        if let Some(ref mut app) = *app_{
                            *app.world.resource_mut(window_scale_factor()) = *scale_factor;
                            tracing::info!("scale_factor sc{:?}",*scale_factor);
                            app.handle_static_event(
                                &Event::WindowEvent {
                                    window_id: *window_id,
                                    event: WindowEvent::Resized(**new_inner_size),
                                },
                                control_flow,
                            );
                        }

                    } else if let Some(event) = event.to_static() {

                        if let Event::WindowEvent {window_id,  event } = event.clone(){
                            match event{
                                WindowEvent::Destroyed =>{
                                    *control_flow = ControlFlow::Exit;
                                }
                                _=>{}
                            }
                        }
                        let app_ = self.app.clone();
                        let mut app_ = app_.lock();
                        if let Some(ref mut app) = *app_{
                            app.handle_static_event(&event, control_flow);
                        }

                    }
                }

            });
        } else {
            // Fake event loop in headless mode
            // loop {
            //     let mut control_flow = ControlFlow::default();
            //     //let exit_status =
            //         //self.handle_static_event(&Event::MainEventsCleared, &mut control_flow);
            //     if control_flow == ControlFlow::Exit {
            //         //return exit_status;
            //     }
            // }
        }
    }
    pub fn run_with_view(&mut self,init: impl for<'x> AsyncInit<'x>  +Copy+ Clone+Send+'static){
        let i_c = init.clone();
        let app_c = self.app.clone();
        let rt = self.runtime.clone();
        std::thread::spawn(move||{
            //self.runtime.block_on(async move{
               rt.block_on(async move{
                if let Some(app ) =app_c.lock().as_mut(){
                    i_c.call(app).await;
                }
                use std::time::{Duration};
                use std::thread::sleep;
                loop{
                    sleep(Duration::new(5,0));
                }
            });
        });
    }
}
pub struct App {
    pub world: World,
    pub ctl_rx: flume::Receiver<WindowCtl>,
    pub systems: SystemGroup,
    pub gpu_world_sync_systems: SystemGroup<GpuWorldSyncEvent>,
    pub window_event_systems: SystemGroup<Event<'static, ()>>,
    pub runtime: RuntimeHandle,
    pub window: Option<Arc<Window>>,
    //event_loop: Option<EventLoop<()>>,
    fps: FpsCounter,
    #[cfg(feature = "profile")]
    _puffin: Option<puffin_http::Server>,
    modifiers: ModifiersState,

    window_focused: bool,
    update_title_with_fps_stats: bool,
    #[cfg(target_os = "unknown")]
    _drop_handles: Vec<Box<dyn std::fmt::Debug>>,
    current_time: Instant,

    #[cfg(target_os = "unknown")]
    force_resize_event_rx: flume::Receiver<(u32, u32)>,
    //view:Option<Object>
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("App");
        d.field("world", &self.world)
            .field("systems", &self.systems)
            .field("gpu_world_sync_systems", &self.gpu_world_sync_systems)
            .field("window_event_systems", &self.window_event_systems)
            .field("runtime", &self.runtime)
            .field("window", &self.window)
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
        event: &Event<'static, ()>,
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
            *control_flow = ControlFlow::Exit;
            return ExitStatus::SUCCESS
        }
        match event {
            Event::MainEventsCleared => {
                let frame_start = Instant::now();
                let external_time = frame_start.duration_since(self.current_time);
                // Handle window control events
                for v in self.ctl_rx.try_iter() {
                    tracing::trace!(?v, "window control");
                    match v {
                        WindowCtl::GrabCursor(mode) => {
                            if let Some(window) = &self.window {
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
                            if let Some(window) = &self.window {
                                window.set_cursor_visible(show);
                            }
                        }
                        WindowCtl::SetCursorIcon(icon) => {
                            if let Some(window) = &self.window {
                                window.set_cursor_icon(icon);
                            }
                        }
                        WindowCtl::SetTitle(title) => {
                            if let Some(window) = &self.window {
                                window.set_title(&title);
                            }
                        }
                        WindowCtl::SetFullscreen(fullscreen) => {
                            if let Some(window) = &self.window {
                                window.set_fullscreen(if fullscreen {
                                    Some(Fullscreen::Borderless(None))
                                } else {
                                    None
                                });
                            }
                        }
                        WindowCtl::ExitProcess(exit_status) => {
                            *control_flow = ControlFlow::Exit;
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

                if let Some(fps) = self.fps.frame_next() {
                    world
                        .set(world.resource_entity(), self::fps_stats(), fps.clone())
                        .unwrap();
                    if self.update_title_with_fps_stats {
                        if let Some(window) = &self.window {
                            window.set_title(&format!(
                                "{} [{}, {} entities]",
                                world.resource(window_title()),
                                fps.dump_both(),
                                world.len()
                            ));
                        }
                    }
                }

                if let Some(window) = &self.window {
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
                    gpu.resize(*size);

                    let size = uvec2(size.width, size.height);
                    if let Some(window) = &self.window {
                        let scale_factor = window.scale_factor();
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
                }
                WindowEvent::CloseRequested => {
                    tracing::info!("Closing...");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            if let VirtualKeyCode::Q = keycode {
                                if self.modifiers.logo() {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                        }
                    }
                }
                WindowEvent::ModifiersChanged(state) => {
                    self.modifiers = *state;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if self.window_focused {
                        let p = vec2(position.x as f32, position.y as f32)
                            / self
                                .window
                                .as_ref()
                                .map(|x| x.scale_factor() as f32)
                                .unwrap_or(1.);
                        world
                            .set(world.resource_entity(), cursor_position(), p)
                            .unwrap();
                    }
                }
                WindowEvent::Destroyed => {
                    *control_flow = ControlFlow::Exit;
                }
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

#[derive(Debug)]
pub struct ExamplesSystem;
impl System<Event<'static, ()>> for ExamplesSystem {
    #[allow(clippy::single_match)]
    fn run(&mut self, world: &mut World, event: &Event<'static, ()>) {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_keycode),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_keycode {
                VirtualKeyCode::F1 => dump_world_hierarchy_to_user(world),
                #[cfg(not(target_os = "unknown"))]
                VirtualKeyCode::F2 => world.dump_to_tmp_file(),
                #[cfg(not(target_os = "unknown"))]
                VirtualKeyCode::F3 => world.resource(main_renderer()).lock().dump_to_tmp_file(),
                _ => {}
            },
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct MeshBufferUpdate;
impl System for MeshBufferUpdate {
    fn run(&mut self, world: &mut World, _event: &FrameEvent) {
        profiling::scope!("MeshBufferUpdate.run");
        let assets = world.resource(asset_cache()).clone();
        let gpu = world.resource(gpu()).clone();
        let mesh_buffer = MeshBufferKey.get(&assets);
        let mut mesh_buffer = mesh_buffer.lock();
        mesh_buffer.update(&gpu);
    }
}

#[derive(Debug)]
pub struct DummySystem;
impl System for DummySystem {
    fn run(&mut self, _world: &mut World, _event: &FrameEvent) {}
}
