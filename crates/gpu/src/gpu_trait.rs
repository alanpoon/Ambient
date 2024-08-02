use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window};
use std::sync::{Mutex,Arc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window,
};
use wgpu::{ColorTargetState, ColorWrites, InstanceFlags, PipelineCompilationOptions, PresentMode, SurfaceConfiguration, TextureViewDescriptor};
use crate::gpu::{self, Renderer, SurfaceState};
pub trait GpuTrait{
    //fn surface(&self)->&Option<wgpu::Surface<'static>>;
    fn device(&self)->Arc<wgpu::Device>;
    fn queue(&self)->Arc<wgpu::Queue>;
    //fn swapchain_format(&self)->Option<TextureFormat>;
    //fn adapter(&self)->wgpu::Adapter;
    fn will_be_polled(&self)->bool;
    //fn view(&self)->Option<Arc<Window>>;
}
const SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;
impl crate::gpu::Gpu{
    async fn create_renderer(&mut self) {
        let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options:PipelineCompilationOptions::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: self.surface_state.as_ref().unwrap().view_format,
                    blend: None,
                    write_mask: ColorWrites::all(),
                })],
                compilation_options:PipelineCompilationOptions::default()
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache:None
        });

        self.renderer = Some(Renderer {
            render_pipeline,
        });
    }

    fn setup_swapchain(&self, size: PhysicalSize<u32>) {
        let surface_state = self.surface_state.as_ref().unwrap();
        let surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_state.view_format,
            width:size.width,
            //width: size.width * scale_factor as u32,
            //height: size.height ,
            height:2048,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_state.alpha_mode,
            view_formats: vec![surface_state.view_format],
            desired_maximum_frame_latency:60
        };
        surface_state.surface.configure(&self.device, &surface_configuration);
    }

    pub fn resumed(&mut self) {
        let window = self.window.clone().unwrap();
        let surface = unsafe {
            self.instance.create_surface(window.clone())
        }.unwrap();
        // let surface: wgpu::Surface =  self.instance.create_surface(&window).unwrap();
        let cap = surface.get_capabilities(&self.adapter);
        self.surface_state = Some(SurfaceState {
            surface,
            view_format: cap.formats[0],
            alpha_mode: cap.alpha_modes[0],
        });

        self.setup_swapchain(window.inner_size());
        //pollster::block_on(self.create_renderer());
        futures::executor::block_on(self.create_renderer());
    }

    pub fn suspended(&mut self) {
        self.renderer.take();
        self.surface_state.take();
    }

    pub fn resize(&self, window_size: PhysicalSize<u32>) {
        self.setup_swapchain(window_size);
    }

    pub fn render(&self) {
        if let (Some(surface_state), Some(renderer)) = (&self.surface_state, &self.renderer) {
            let render_texture = surface_state.surface.get_current_texture().unwrap();
            let render_texture_view = render_texture.texture.create_view(&TextureViewDescriptor::default());

            let mut encoder = self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let t = (self.last_time.elapsed().as_secs_f64() / 5.0).sin();
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &render_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: t,
                                b: 1.0 - t,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes:None,
                    occlusion_query_set:None
                });
                rpass.set_pipeline(&renderer.render_pipeline);
                rpass.draw(0..3, 0..1);
            }

            self.queue.submit(Some(encoder.finish()));

            render_texture.present();
        }
    }
}

impl GpuTrait for Arc<Mutex<gpu::Gpu>>{
    // fn surface(&self)->&Option<wgpu::Surface<'static>>{
    //     &(self.lock().unwrap().surface)
    // }
    fn device(&self)->Arc<wgpu::Device>{
        self.lock().unwrap().device.clone()
    }
    fn queue(&self)->Arc<wgpu::Queue>{
        self.lock().unwrap().queue.clone()
    }
    // fn swapchain_format(&self)->Option<TextureFormat>{
    //     self.lock().unwrap().swapchain_format
    // }
    // fn adapter(&self)->wgpu::Adapter{
    //     self.lock().unwrap().adapter
    // }
    fn will_be_polled(&self)->bool{
        self.lock().unwrap().will_be_polled
    }
    // fn view(&self)->Option<Arc<Window>>{
    //     self.lock().unwrap().window
    // }
}
