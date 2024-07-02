use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use ambient_app::{fps_stats, window_title, AppBuilder};
use ambient_audio::{AudioMixer, AudioStream};
use ambient_cameras::UICamera;
use ambient_client_shared::game_view::GameView;
use ambient_core::{
    asset_cache, gpu, runtime,
    window::{window_ctl, ExitStatus, WindowCtl},
};
use ambient_ecs::{Entity, SystemGroup, World};
use ambient_element::{
    consume_context, element_component, use_effect, use_ref_with, use_spawn, use_state,
    use_state_with, Element, ElementComponentExt, Group, Hooks,
};
use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt,SyncAssetKey},
    cb,asset_url::{ContentBaseUrlKey, UsingLocalDebugAssetsKey},
    download_asset::ReqwestClientKey,
};
use ambient_network::{
    client::{client_network_stats, GameClientRenderTarget},
    hooks::use_remote_resource,
    native::client::{ ResolvedAddr},
};
use ambient_settings::SettingsKey;
use once_cell::sync::Lazy;
use ambient_sys::{task::RuntimeHandle, time::Instant};
use ambient_ui_native::{Dock, WindowSized};
use glam::uvec2;
use tokio::runtime;
use winit::event_loop::{EventLoop,EventLoopBuilder};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
use crate::{
    //cli::{ClientCli, GoldenImageCommand},
    shared::{self},

        //certs::CERT},
};
mod package_path;
mod wasm;
use ambient_core::{
    camera::active_camera,
    main_scene,
    transform::{scale, translation},
};
use ambient_native_std::math::SphericalCoords;
use ambient_primitives::{Cube, Quad};
use ambient_renderer::{cast_shadows, color, outline};
use glam::{vec3, vec4, Vec3, Vec4};
use ambient_app::App;
use std::ffi::CString;
use std::io::{BufRead, BufReader};
//use quinn::Connection;
pub const QUIC_INTERFACE_PORT: u16 = 9000;
#[derive(Debug)]
pub struct CertKey;
impl SyncAssetKey<Option<Vec<u8>>> for CertKey {
    fn load(&self, _assets: ambient_native_std::asset_cache::AssetCache) -> Option<Vec<u8>> {
       None
    }
}
#[cfg(target_os = "android")]
pub async fn init(app:&mut App,android_app:AndroidApp){
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
#[cfg(target_os = "android")]
pub async fn init2(app: &mut App,android_app:AndroidApp) {
    //let runtime = app.world.resource(runtime()).clone();
    // let server_addr = runtime.block_on().unwrap();
    tracing::info!("init....");
    let assets = app.world.resource(asset_cache()).clone();
      let asset_manager = android_app.asset_manager();
    let cert_asset = asset_manager
        .open(&CString::new("localhost.crt").unwrap())
        .expect("Could not open asset");
    let cert = Some(BufReader::new(cert_asset).fill_buf().unwrap().to_vec());

    tracing::info!("cert....{:?}",cert);
    let server_addr= async move {
        //let Some(mut host) = Some(String::from("eu.proxy.ambient.run:9131")) else {
        let Some(mut host) = Some(String::from("localhost:4433")) else {
            return Ok(ResolvedAddr::localhost_with_port(QUIC_INTERFACE_PORT));
        };

        if host.starts_with("http://") || host.starts_with("https://") {
            tracing::info!("NOTE: Joining server by http url is still experimental and can be removed without warning.");
            let assets_ref = &assets;
            let reqwest = &ReqwestClientKey.get(assets_ref);
            host = reqwest.get(host).send().await?.text().await?;

            if host.is_empty() {
                anyhow::bail!("Failed to resolve host");
            }
        }
        if !host.contains(':') {
            host = format!("{host}:{QUIC_INTERFACE_PORT}");
        }
        ResolvedAddr::lookup_host(&host).await
    }.await.unwrap();

    tracing::info!("init");
    let mixer = None;

    let user_id = ambient_client_shared::util::random_username();
    let golden_image_output_dir = None;
    let is_debug = true;
    //34.141.183.28
    // let socket: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10,0 ,2, 2)), 9000);
    // let server_addr = ResolvedAddr{host_name:String::from("localhost"),addr:socket};
    //let server_addr = app.world.resource(server_addr());
    *app.world.resource_mut(window_title()) = "Ambient".to_string();

    #[cfg(feature = "production")]
    let fail_on_version_mismatch = true;

    #[cfg(not(feature = "production"))]
    let fail_on_version_mismatch = false;

    let golden_image = None;

    tracing::info!("after open_connection");
    MainApp {
        server_addr,
        user_id,
        fail_on_version_mismatch,
        show_debug: is_debug,
        golden_image_cmd: golden_image,
        golden_image_output_dir,
        cert,
        mixer,
    }
    .el()
    .spawn_interactive(&mut app.world);

}


#[element_component]
fn TitleUpdater(hooks: &mut Hooks) -> Element {
    let (net, _) = use_remote_resource(hooks, client_network_stats()).expect("No game client");
    tracing::info!("net...{:?}",net);
    let world = &hooks.world;
    let title = world.resource(window_title());
    let fps = world
        .get_cloned(hooks.world.resource_entity(), fps_stats())
        .ok()
        .filter(|f| !f.fps().is_nan());

    let title = match (fps, net) {
        (None, None) => title.clone(),
        (Some(fps), None) => format!("{} [{}]", title, fps.dump_both()),
        (None, Some(net)) => format!("{} [{}]", title, net),
        (Some(fps), Some(net)) => format!("{} [{}, {}]", title, fps.dump_both(), net),
    };
    world
        .resource(window_ctl())
        .send(WindowCtl::SetTitle(title))
        .ok();

    Element::new()
}
use ambient_network::native::client::ClientView;
#[derive(Debug,Clone)]
pub struct GoldenImageCommand{}
#[element_component]
pub fn MainApp(
    hooks: &mut Hooks,
    server_addr: ResolvedAddr,
    golden_image_output_dir: Option<PathBuf>,
    user_id: String,
    fail_on_version_mismatch: bool,
    show_debug: bool,
    golden_image_cmd: Option<GoldenImageCommand>,
    cert: Option<Vec<u8>>,
    mixer: Option<AudioMixer>,
) -> Element {
    let (loaded, set_loaded) = use_state(hooks, false);

    Group::el([
        UICamera.el(),
        ambient_client_shared::player::PlayerRawInputHandler.el(),
        WindowSized::el([ClientView {
            server_addr,
            user_id,
            fail_on_version_mismatch,
            // NOTE: client.game_state is **locked** and accesible through game_state.
            //
            // This is to prevent another thread from updating using the client after connection but
            // just before `on_loaded`. This is a very small window of time, but does occasionally
            // happen, especially when joining a server which is already running and finished
            // loading.
            on_loaded: cb(move |_, game_state| {

                let world = &mut game_state.world;
                let assets = world.resource(asset_cache()).clone();

                wasm::initialize(world, &assets, mixer.clone()).unwrap();

                UICamera.el().spawn_static(world);
                set_loaded(true);
                tracing::info!("on_loaded");
                Ok(Box::new(|| {
                    tracing::info!("Disconnecting client");
                }))
            }),
            systems_and_resources: cb(|| {
                let mut resources = Entity::new();

                let bistream_handlers = HashMap::new();
                resources.set(
                    ambient_network::client::bi_stream_handlers(),
                    bistream_handlers,
                );

                let unistream_handlers = HashMap::new();
                resources.set(
                    ambient_network::client::uni_stream_handlers(),
                    unistream_handlers,
                );

                let dgram_handlers = HashMap::new();
                resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

                (systems(), resources)
            }),
            cert,
            create_rpc_registry: cb(shared::create_server_rpc_registry),
            inner: Dock::el(vec![
                TitleUpdater.el(),
                Element::new(),
                GameView { show_debug }.el(),
            ]),
        }
        .el()]),
    ])
}
// async fn MainApp2(
//     world: &mut World,
//     server_addr: ResolvedAddr,
//     golden_image_output_dir: Option<PathBuf>,
//     user_id: String,
//     fail_on_version_mismatch: bool,
//     show_debug: bool,
//     golden_image_cmd: Option<String>,
//     cert: Option<Vec<u8>>,
//     mixer: Option<AudioMixer>,
// )  {
//     tracing::info!("MainApp--");
//     //let (loaded, set_loaded) = use_state(hooks, false);

//     ClientView {
//         server_addr,
//         user_id,
//         fail_on_version_mismatch,
//         // NOTE: client.game_state is **locked** and accesible through game_state.
//         //
//         // This is to prevent another thread from updating using the client after connection but
//         // just before `on_loaded`. This is a very small window of time, but does occasionally
//         // happen, especially when joining a server which is already running and finished
//         // loading.
//         on_loaded: cb(move |_, game_state| {
//             tracing::info!("on_loaded--");
//             let world = &mut game_state.world;
//             let assets = world.resource(asset_cache()).clone();

//             wasm::initialize(world, &assets, mixer.clone()).unwrap();

//             UICamera.el().spawn_static(world);
//             //set_loaded(true);

//             Ok(Box::new(|| {
//                 tracing::info!("Disconnecting client");
//             }))
//         }),
//         systems_and_resources: cb(|| {
//             let mut resources = Entity::new();

//             let bistream_handlers = HashMap::new();
//             resources.set(
//                 ambient_network::client::bi_stream_handlers(),
//                 bistream_handlers,
//             );

//             let unistream_handlers = HashMap::new();
//             resources.set(
//                 ambient_network::client::uni_stream_handlers(),
//                 unistream_handlers,
//             );

//             let dgram_handlers = HashMap::new();
//             resources.set(ambient_network::client::datagram_handlers(), dgram_handlers);

//             (systems(), resources)
//         }),
//         cert,
//         create_rpc_registry: cb(shared::create_server_rpc_registry),
//         inner: Dock::el(vec![
//             //TitleUpdater.el(),
//             // if let Some(golden_image_cmd) = golden_image_cmd.filter(|_| loaded) {
//             //     GoldenImageTest::el(golden_image_output_dir, golden_image_cmd)
//             // } else {
//                 Element::new()
//             //}
//             ,
//             GameView { show_debug }.el(),
//         ]),        }.init(world).await;
// }

#[element_component]
fn GoldenImageTest(
    hooks: &mut Hooks,
    golden_image_output_dir: Option<PathBuf>,
    golden_image_cmd: String,
) -> Element {
    let (render_target, _) = consume_context::<GameClientRenderTarget>(hooks).unwrap();
    let render_target_ref = use_ref_with(hooks, |_| render_target.clone());
    *render_target_ref.lock() = render_target.clone();
    let golden_image_output_dir = golden_image_output_dir.unwrap_or(PathBuf::new());
    let screenshot_path = golden_image_output_dir.join("screenshot.png");
    let fail_screenshot_path = golden_image_output_dir.join("fail_screenshot.png");
    let (old_screenshot, _) = use_state_with(hooks, |_| {
        tracing::info!("Loading screenshot from {:?}", screenshot_path);
        Some(Arc::new(image::open(&screenshot_path).ok()?))
    });




    Element::new()
}

fn systems() -> SystemGroup {
    SystemGroup::new(
        "client",
        vec![
            Box::new(ambient_prefab::systems()),
            Box::new(ambient_decals::client_systems()),
            Box::new(ambient_primitives::systems()),
            Box::new(ambient_sky::systems()),
            Box::new(ambient_water::systems()),
            Box::new(ambient_gizmos::client_systems()),
            Box::new(wasm::systems()),
            Box::new(ambient_client_shared::player::systems_final()),
        ],
    )
}
