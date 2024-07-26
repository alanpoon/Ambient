use std::sync::Arc;

use ambient_native_std::asset_cache::SyncAssetKey;
use ambient_settings::RenderSettings;
use anyhow::Context;
use bytemuck::{Pod, Zeroable};
use glam::{uvec2, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use wgpu::{InstanceDescriptor, InstanceFlags, Limits, PresentMode, TextureFormat};
use app_surface::SurfaceDeviceQueue;
use winit::window::Window;
// #[cfg(debug_assertions)]
pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
#[derive(Debug)]
pub struct GpuKey;
impl SyncAssetKey<Arc<Gpu>> for GpuKey {}


#[derive(Debug)]
pub struct Gpu {
    pub surface: Option<wgpu::Surface<'static>>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub swapchain_format: Option<TextureFormat>,
    pub swapchain_mode: Option<PresentMode>,
    pub adapter: wgpu::Adapter,
    /// If this is true, we don't need to use blocking device.polls, since they are assumed to be polled elsewhere
    pub will_be_polled: bool,
    pub view:Option<Arc<Window>>,
}
impl Gpu{
    pub async fn new(sdq: Option<SurfaceDeviceQueue>,window:Option<Arc<Window>>)->anyhow::Result<Self>{
        if let Some(sdq) = sdq{

            Ok(Gpu{
                surface:Some(sdq.surface),
                device:sdq.device,
                swapchain_format:None,
                swapchain_mode: None,
                adapter:sdq.adapter,
                queue:sdq.queue,
                will_be_polled:true,
                view:window
            })
        }else{
            let backends = if cfg!(target_os = "windows") {
                wgpu::Backends::VULKAN
            } else if cfg!(target_os = "android") {
                wgpu::Backends::VULKAN
            } else if cfg!(target_os = "macos") {
                wgpu::Backends::PRIMARY
            } else if cfg!(target_os = "unknown") {
                wgpu::Backends::BROWSER_WEBGPU
            }else if cfg!(target_os = "ios") {
                wgpu::Backends::METAL
            }else if cfg!(target_os = "ios-sim") {
                wgpu::Backends::METAL
            } else {
                wgpu::Backends::all()
            };

            let instance = wgpu::Instance::new(InstanceDescriptor {
                backends,
                // NOTE: Vulkan is used for windows as a non-zero indirect `first_instance` is not supported, and we have to resort direct rendering
                // See: <https://github.com/gfx-rs/wgpu/issues/2471>
                //
                // TODO: upgrade to Dxc? This requires us to ship additionall dll files, which may be
                // possible using an installer. Nevertheless, we are currently using Vulkan on windows
                // due to `base_instance` being broken on windows.
                // https://docs.rs/wgpu/latest/wgpu/enum.Dx12Compiler.html
                dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
                // dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
                //     dxil_path: Some("./dxil.dll".into()),
                //     dxc_path: Some("./dxcompiler.dll".into()),
                // },
                flags:InstanceFlags::all(),
                gles_minor_version:wgpu::Gles3MinorVersion::Automatic
            });


            let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::util::power_preference_from_env()
                    .unwrap_or(wgpu::PowerPreference::HighPerformance),
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("No suitable GPU adapters found on the system!");
            let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features:adapter.features(),
                    required_limits:adapter.limits(),
                    memory_hints:wgpu::MemoryHints::Manual { suballocated_device_memory_block_size: 1..2 }
                },
                None,
            )
            .await
            .context("Failed to create device")?;
            Ok(Gpu{
                surface:None,
                device:Arc::new(device),
                swapchain_format:None,
                swapchain_mode: None,
                adapter:adapter,
                queue:Arc::new(queue),
                will_be_polled:true,
                view:window
            })
        }

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
            desired_maximum_frame_latency:1
        }
    }
    pub fn get_view(&self) -> Option<&Arc<Window>> {
        return self.view.as_ref();
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
