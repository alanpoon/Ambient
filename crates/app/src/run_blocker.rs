use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Fullscreen, Window, WindowBuilder},
};
use crate::{App, AppBuilder};
use std::{ops::DerefMut, sync::{Arc,Mutex}};
use ambient_core::{
    window::{
        window_scale_factor,
    },
};
use glam::{uvec2, vec2, IVec2, UVec2, Vec2};
use ambient_sys::task::RuntimeHandle;
use ambient_settings::SettingsKey;
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt},
    fps_counter::{FpsCounter, FpsSample},
};
use crate::{AsyncInit};
use futures::Future;
pub struct AppWrapper <'x>{
    app:Arc<Mutex<Option<App>>>,
    init: Arc<dyn AsyncInit<'x, Future = Box<dyn Future<Output = ()>>> + 'x>,
}
impl <'a> AppWrapper<'a>{
    pub fn new(init: dyn AsyncInit<'x, Future = Box<dyn Future<Output = ()>>>)->Self{
        AppWrapper{
            app:Arc::new(Mutex::new(None)),
            init:Arc::new(init)
        }
    }
    pub fn run_blocking(mut self,event_loop:EventLoop<()>,) {
            let runtime = RuntimeHandle::current();
            let assets = AssetCache::new(runtime.clone());

            let settings = SettingsKey.get(&assets);
            let running = Arc::new(Mutex::new(false));
            let headless:Option<()> = None;
            let (window, event_loop) = if headless.is_some() {
                (None, None)
            } else {
                let event_loop = event_loop;
                let window = WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
                    width: settings.render.resolution().0,
                    height: settings.render.resolution().1,
                });
                let window_position_override :Option<IVec2> = None;
                let window_size_override:Option<UVec2> = None;
                let window = if let Some(position) = window_position_override {
                    window.with_position(winit::dpi::LogicalPosition {
                        x: position.x,
                        y: position.y,
                    })
                } else {
                    window
                };
                let window = if let Some(size) = window_size_override {
                    window.with_inner_size(winit::dpi::LogicalSize {
                        width: size.x,
                        height: size.y,
                    })
                } else {
                    window
                };
                let window = Arc::new(window.build(&event_loop).unwrap());
                (Some(window), Some(event_loop))
            };
            tracing::debug!("window is some {:?} event_loop is {:?}",window.is_some(),event_loop.is_some());
            // Clone a reference to the Mutex for the event loop
            let running_clone = Arc::clone(&running);
            if let Some(event_loop) = event_loop{
                let init_c = self.init.clone();
                
                event_loop.run(move |event, _, control_flow| {
                    if let Event::Resumed = event{
                        *running.lock().unwrap() = true;
                        let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
                        let app_c = self.app.clone();
                        let window_c= window.clone();
                        rt.block_on(async move {
                            let mut app = AppBuilder::simple_dual().build(window_c).await.unwrap();
                            init_c.call(&mut app).await;
                            *app_c.lock().unwrap() = Some(app);
                        });
                        
    
                    }else if *running_clone.lock().unwrap() {
                        let mut app_c = self.app.lock().unwrap();
                        let mut app_c_c = app_c.deref_mut();
                        if let  Some( ref mut app) = app_c_c{
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
                            *app.world.resource_mut(window_scale_factor()) = *scale_factor;
                            app.handle_static_event(
                                &Event::WindowEvent {
                                    window_id: *window_id,
                                    event: WindowEvent::Resized(**new_inner_size),
                                },
                                control_flow,
                            );
                        } else if let Some(event) = event.to_static() {
                            app.handle_static_event(&event, control_flow);
                        }
                        }
                        
                    }
                    
                });
            }
            
       
    }
}