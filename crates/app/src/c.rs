//use crate::run_blocker::AppWrapper;

use super::{App, AppBuilder,AppWrapper};
use ambient_core::{
    camera::active_camera,
    main_scene,
    transform::{scale, translation},
};
use ambient_element::ElementComponentExt;
use ambient_native_std::math::SphericalCoords;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::{cast_shadows, color, outline};
use glam::{vec3, vec4, Vec3, Vec4};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window,
};

async fn init(app: &mut App) {
    let world = &mut app.world;

    Cube.el()
        .with(color(), vec4(0.5, 0.5, 0.5, 1.))
        .with(translation(), Vec3::Z)
        .with(cast_shadows(), ())
        .with(outline(), Vec4::ONE)
        .spawn_static(world);
    Quad.el().with(scale(), Vec3::ONE * 10.).spawn_static(world);

    ambient_cameras::spherical::new(
        vec3(0., 0., 0.),
        SphericalCoords::new(std::f32::consts::PI / 4., std::f32::consts::PI / 4., 5.),
    )
    .with(active_camera(), 0.)
    .with(main_scene(), ())
    .spawn(world);
}
use crate::AsyncInit;

pub fn run(mut event_loop: EventLoop<()>) {
   
    // wgpu_subscriber::initialize_default_subscriber(None);
    //AppBuilder::simple_dual().with_event_loop(event_loop).block_on(init);
    //AppWrapper::new(init).run_blocking(event_loop);
    AppWrapper::new_with_event_loop(event_loop).run_blocking(init);
}
