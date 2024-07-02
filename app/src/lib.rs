use ambient_native_std::{
    asset_cache::{AssetCache, SyncAssetKeyExt,SyncAssetKey},
    asset_url::{ContentBaseUrlKey, UsingLocalDebugAssetsKey},
    download_asset::AssetsCacheOnDisk,
    download_asset::ReqwestClientKey,
};
use ambient_settings::SettingsKey;

pub mod client;
mod shared;

use ambient_physics::physx::PhysicsKey;
use anyhow::Context;
use serde::Deserialize;
use winit::event_loop::EventLoop;
use std::path::Path;
use ambient_network::native::client::ResolvedAddr;
use std::path::PathBuf;
use ambient_audio::AudioStream;
use ambient_app::AppWrapper;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
pub fn new_main(eventloop:EventLoop<()>,android_app:AndroidApp){
    tracing::info!("start main..");

    //let _settings = SettingsKey.get(&assets);
    //34.141.183.28
    //let socket: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(34, 141 ,183, 28)), 9167);

    //let rt = ambient_sys::task::make_native_multithreaded_runtime().unwrap();
    ambient_git_rev_init::init().expect("Should be called exactly once");

    shared::components::init().unwrap();

    // let runtime = rt.handle();
    // let assets = AssetCache::new(runtime.clone());
    // PhysicsKey.get(&assets); // Load physics
    //AssetsCacheOnDisk.insert(&assets, false); // Disable disk caching for now; see https://github.com/AmbientRun/Ambient/issues/81
    let audio_bool = false;
    let audio_stream = if audio_bool {
        match AudioStream::new() {
            Ok(v) => Some(v),
            Err(err) => {
                tracing::error!("Failed to initialize audio stream: {err}");
                None
            }
        }
    } else {
        None
    };
    //let mixer = audio_stream.as_ref().map(|v| v.mixer().clone());

    //let settings = SettingsKey.get(&assets);
    //let is_debug: bool = std::env::var("AMBIENT_DEBUGGER").is_ok() || args.debugger;
    let is_debug = false;
    //box_init();

    tracing::info!("before event_loop");
    let  aw = AppWrapper::new_with_event_loop(eventloop);

    tracing::info!("after event_loop");


    let status = aw.run_blocking(client::init2,android_app,Box::new(||{
        shared::components::init().unwrap();

    }));


}
