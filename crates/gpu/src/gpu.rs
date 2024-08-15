use std::sync::Arc;

use ambient_native_std::asset_cache::SyncAssetKey;
use ambient_settings::RenderSettings;
use anyhow::Context;
use bytemuck::{Pod, Zeroable};
use glam::{uvec2, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use wgpu::{InstanceDescriptor, PresentMode, TextureFormat};
use winit::window::Window;
//use core_graphics::{base::CGFloat, geometry::CGRect};
pub struct CGRect{
    pub size_width: u32,
    pub size_height: u32
}
use objc::*;
use objc::runtime::Object;
use libc::c_void;
// #[cfg(debug_assertions)]
pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
// #[cfg(not(debug_assertions))]
// pub const DEFAULT_SAMPLE_COUNT: u32 = 4;
use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle,RawDisplayHandle};
pub struct WrapWindow(pub *mut c_void);
unsafe impl HasRawWindowHandle for WrapWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = AppKitWindowHandle::empty();
        handle.ns_window = self.0 as *mut std::ffi::c_void;
        RawWindowHandle::AppKit(handle)
    }
}
unsafe impl HasRawDisplayHandle for WrapWindow {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        let mut handle = AppKitDisplayHandle::empty();
        //handle.ns_view = self.0 as *mut std::ffi::c_void;
        RawDisplayHandle::AppKit(handle)
    }
}
#[derive(Debug)]
pub struct GpuKey;
impl SyncAssetKey<Arc<Gpu>> for GpuKey {}

#[derive(Debug)]
pub struct Gpu {
    pub surface: Option<wgpu::Surface>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_format: Option<TextureFormat>,
    pub swapchain_mode: Option<PresentMode>,
    pub adapter: wgpu::Adapter,
    /// If this is true, we don't need to use blocking device.polls, since they are assumed to be polled elsewhere
    pub will_be_polled: bool,
}

impl Gpu {
    pub async fn new(window: Option<&Window>) -> anyhow::Result<Self> {
        Self::with_config(window, false, &RenderSettings::default()).await
    }
    pub async fn with_view(view:Option<*mut Object>, metal_layer:Option<*mut c_void>, will_be_polled: bool,settings:&RenderSettings) ->anyhow::Result<Self> {
        let backends = if cfg!(target_os = "windows") {
            wgpu::Backends::VULKAN
        } else if cfg!(target_os = "android") {
            wgpu::Backends::VULKAN
        } else if cfg!(target_os = "macos") {
            wgpu::Backends::PRIMARY
        } else if cfg!(target_os = "unknown") {
            wgpu::Backends::BROWSER_WEBGPU
        } else if cfg!(target_os = "ios") {
            wgpu::Backends::METAL
        } else if cfg!(target_os = "ios-sim") {
            wgpu::Backends::METAL
        } else {
            wgpu::Backends::all()
        };
        println!("backends{:?}",backends);
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            // NOTE: Vulkan is used for windows as a non-zero indirect `first_instance` is not supported, and we have to resort direct rendering
            // See: <https://github.com/gfx-rs/wgpu/issues/2471>
            //
            // TODO: upgrade to Dxc? This requires us to ship additionall dll files, which may be
            // possible using an installer. Nevertheless, we are currently using Vulkan on windows
            // due to `base_instance` being broken on windows.
            // https://docs.rs/wgpu/latest/wgpu/enum.Dx12Compiler.html
            //dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc{
                dxil_path:None,
                dxc_path:None,
            },
            // dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
            //     dxil_path: Some("./dxil.dll".into()),
            //     dxc_path: Some("./dxcompiler.dll".into()),
            // },
        });

        let surface = metal_layer
            .map(|window| unsafe { instance.create_surface_from_core_animation_layer(&window) })
            .transpose()
            .context("Failed to create surface")?;
        // let surface = unsafe {
        //     instance
        //         .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
        //             obj.metal_layer,
        //         ))
        //         .expect("Surface creation failed")
        // };
        {
            tracing::debug!("Available adapters:");
            for adapter in instance.enumerate_adapters(wgpu::Backends::METAL) {
                println!("Adapter: {:?}", adapter.get_info());
            }
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find an appopriate adapter")?;

        tracing::info!("Using gpu adapter: {:?}", adapter.get_info());

        tracing::info!("Adapter features:\n{:#?}", adapter.features());
        let adapter_limits: wgpu::Limits = adapter.limits();
        tracing::info!("Adapter limits:\n{:#?}", adapter_limits);

        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                // The renderer will dispatch 1 indirect draw command for *each* primitive in the
                // scene, but the data draw data such as index_count, first_instance, etc lives on
                // the gpu
                let features = wgpu::Features::empty();
            } else if #[cfg(target_os = "android")]{
                let features = wgpu::Features::empty();
            } else if #[cfg(target_os = "unknown")] {

                // Same as above, but the *web*gpu target requires a feature flag to be set, or
                // else indirect commands no-op
                let features = wgpu::Features::INDIRECT_FIRST_INSTANCE;
            } else {
                // TODO: make configurable at runtime
                // The renderer will use indirect drawing with the draw commands *and* count
                // fetched from gpu side buffers
                let features =
                wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
            }
        };

        tracing::info!("Using device features: {features:?}");

        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter.features(),
                limits:adapter.limits(),
            },
            None,
        )
        .await.unwrap();
        //.context("Failed to create device")?;
        tracing::debug!("Device limits:\n{:#?}", device.limits());

        let swapchain_format = surface
            .as_ref()
            .map(|surface| surface.get_capabilities(&adapter).formats[0]);

        tracing::debug!("Swapchain format: {swapchain_format:?}");

        let swapchain_mode = if surface.is_some() {
            if settings.vsync() {
                // From wgpu docs:
                // "Chooses FifoRelaxed -> Fifo based on availability."
                Some(PresentMode::AutoVsync)
            } else {
                // From wgpu docs:
                // "Chooses Immediate -> Mailbox -> Fifo (on web) based on availability."
                Some(PresentMode::AutoNoVsync)
            }
        } else {
            None
        };
        let view = view.unwrap();
        let scale_factor = get_scale_factor(view);
        tracing::debug!("Swapchain present mode: {swapchain_mode:?}");

        if let (Some(surface), Some(mode), Some(format)) =
            (&surface, swapchain_mode, swapchain_format)
        {
            let size = get_view_size(view,scale_factor);
            surface.configure(
                &device,
                &Self::create_sc_desc(format, mode, uvec2(size.0, size.1)),
            );
        }

        Ok(Self {
            device,
            surface,
            queue,
            swapchain_format,
            swapchain_mode,
            adapter,
            will_be_polled,
        })
    }
    pub async fn with_config(
        window: Option<&Window>,
        will_be_polled: bool,
        settings: &RenderSettings,
    ) -> anyhow::Result<Self> {
        let _span = tracing::info_span!("create_gpu").entered();
        // From: https://github.com/KhronosGroup/Vulkan-Loader/issues/552
        #[cfg(not(target_os = "unknown"))]
        {
            std::env::set_var("DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1", "1");
            std::env::set_var("DISABLE_LAYER_NV_OPTIMUS_1", "1");
        }

        let backends = if cfg!(target_os = "windows") {
            wgpu::Backends::VULKAN
        } else if cfg!(target_os = "android") {
            wgpu::Backends::VULKAN
        } else if cfg!(target_os = "macos") {
            wgpu::Backends::PRIMARY
        } else if cfg!(target_os = "unknown") {
            wgpu::Backends::BROWSER_WEBGPU
        } else if cfg!(target_os = "ios") {
            wgpu::Backends::METAL
        } else if cfg!(target_os = "ios-sim") {
            wgpu::Backends::METAL
        } else {
            wgpu::Backends::all()
        };
        println!("backends{:?}",backends);
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            // NOTE: Vulkan is used for windows as a non-zero indirect `first_instance` is not supported, and we have to resort direct rendering
            // See: <https://github.com/gfx-rs/wgpu/issues/2471>
            //
            // TODO: upgrade to Dxc? This requires us to ship additionall dll files, which may be
            // possible using an installer. Nevertheless, we are currently using Vulkan on windows
            // due to `base_instance` being broken on windows.
            // https://docs.rs/wgpu/latest/wgpu/enum.Dx12Compiler.html
            //dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            dx12_shader_compiler: wgpu::Dx12Compiler::Dxc{
                dxil_path:None,
                dxc_path:None,
            },
            // dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
            //     dxil_path: Some("./dxil.dll".into()),
            //     dxc_path: Some("./dxcompiler.dll".into()),
            // },
        });

        let surface = window
            .map(|window| unsafe { instance.create_surface(window) })
            .transpose()
            .context("Failed to create surface")?;
        tracing::info!("...surface {:?}",surface);
        #[cfg(not(target_os = "unknown"))]
        {
            tracing::debug!("Available adapters:");
            for adapter in instance.enumerate_adapters(wgpu::Backends::PRIMARY) {
                tracing::debug!("Adapter: {:?}", adapter.get_info());
            }
        }

        tracing::debug!("Requesting adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find an appopriate adapter")?;

        tracing::info!("Using gpu adapter: {:?}", adapter.get_info());

        tracing::info!("Adapter features:\n{:#?}", adapter.features());
        let adapter_limits: wgpu::Limits = adapter.limits();
        tracing::info!("Adapter limits:\n{:#?}", adapter_limits);

        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                // The renderer will dispatch 1 indirect draw command for *each* primitive in the
                // scene, but the data draw data such as index_count, first_instance, etc lives on
                // the gpu
                let features = wgpu::Features::empty();
            } else if #[cfg(target_os = "android")]{
                let features = wgpu::Features::empty();
            } else if #[cfg(target_os = "unknown")] {

                // Same as above, but the *web*gpu target requires a feature flag to be set, or
                // else indirect commands no-op
                let features = wgpu::Features::INDIRECT_FIRST_INSTANCE;
            } else {
                // TODO: make configurable at runtime
                // The renderer will use indirect drawing with the draw commands *and* count
                // fetched from gpu side buffers
                let features =
                wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
            }
        };

        tracing::info!("Using device features: {features:?}");
        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: adapter.features(),
                limits:adapter.limits(),
            },
            None,
        )
        .await.unwrap();
        //.context("Failed to create device")?;
        tracing::debug!("Device limits:\n{:#?}", device.limits());

        let swapchain_format = surface
            .as_ref()
            .map(|surface| surface.get_capabilities(&adapter).formats[0]);

        tracing::debug!("Swapchain format: {swapchain_format:?}");

        let swapchain_mode = if surface.is_some() {
            if settings.vsync() {
                // From wgpu docs:
                // "Chooses FifoRelaxed -> Fifo based on availability."
                Some(PresentMode::AutoVsync)
            } else {
                // From wgpu docs:
                // "Chooses Immediate -> Mailbox -> Fifo (on web) based on availability."
                Some(PresentMode::AutoNoVsync)
            }
        } else {
            None
        };

        tracing::debug!("Swapchain present mode: {swapchain_mode:?}");

        if let (Some(window), Some(surface), Some(mode), Some(format)) =
            (window, &surface, swapchain_mode, swapchain_format)
        {
            let size = window.inner_size();
            surface.configure(
                &device,
                &Self::create_sc_desc(format, mode, uvec2(size.width, size.height)),
            );
        }

        Ok(Self {
            device,
            surface,
            queue,
            swapchain_format,
            swapchain_mode,
            adapter,
            will_be_polled,
        })
    }

    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        if let Some(surface) = &self.surface {
            if size.width > 0 && size.height > 0 {
                surface.configure(&self.device, &self.sc_desc(uvec2(size.width, size.height)));
            }
        }
    }
    pub fn swapchain_format(&self) -> TextureFormat {
        self.swapchain_format
            .unwrap_or(TextureFormat::Rgba8UnormSrgb)
    }
    pub fn swapchain_mode(&self) -> PresentMode {
        self.swapchain_mode.unwrap_or(PresentMode::Immediate)
    }
    pub fn sc_desc(&self, size: UVec2) -> wgpu::SurfaceConfiguration {
        Self::create_sc_desc(self.swapchain_format(), self.swapchain_mode(), size)
    }
    fn create_sc_desc(
        format: TextureFormat,
        present_mode: PresentMode,
        size: UVec2,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.x,
            height: size.y,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        }
    }
}

pub trait WgslType: Zeroable + Pod + 'static {
    fn wgsl_type() -> &'static str;
}
impl WgslType for f32 {
    fn wgsl_type() -> &'static str {
        "f32"
    }
}
impl WgslType for i32 {
    fn wgsl_type() -> &'static str {
        "i32"
    }
}
impl WgslType for u32 {
    fn wgsl_type() -> &'static str {
        "u32"
    }
}

impl WgslType for Vec2 {
    fn wgsl_type() -> &'static str {
        "vec2<f32>"
    }
}

impl WgslType for Vec3 {
    fn wgsl_type() -> &'static str {
        "vec3<f32>"
    }
}

impl WgslType for Vec4 {
    fn wgsl_type() -> &'static str {
        "vec4<f32>"
    }
}

impl WgslType for UVec2 {
    fn wgsl_type() -> &'static str {
        "vec2<u32>"
    }
}

impl WgslType for UVec3 {
    fn wgsl_type() -> &'static str {
        "vec3<u32>"
    }
}

impl WgslType for UVec4 {
    fn wgsl_type() -> &'static str {
        "vec4<u32>"
    }
}
pub fn get_scale_factor(obj: *mut Object) -> f32 {
    //let mut _scale_factor: CGFloat = 1.0;
    let mut _scale_factor = 1.0;
    #[cfg(target_os = "macos")]
    unsafe {
        let window: *mut Object = msg_send![obj, window];
        if !window.is_null() {
            _scale_factor = msg_send![window, backingScaleFactor];
        }
    };

    {
        _scale_factor = unsafe { msg_send![obj, contentScaleFactor] };
    }

    _scale_factor as f32
}

#[cfg(not(target_os="ios"))]
pub fn get_view_size(view:*mut Object,scale_factor: f32)-> (u32, u32){
   (500,500)
}
#[cfg(target_os="ios")]
pub fn get_view_size(view:*mut Object,scale_factor: f32)-> (u32, u32){
    use core_graphics::{base::CGFloat, geometry::CGRect};
    let s: CGRect = unsafe { msg_send![view, frame] };
    (
        (s.size.width as f32 * scale_factor) as u32,
        (s.size.height as f32 * scale_factor) as u32,
    )
}
