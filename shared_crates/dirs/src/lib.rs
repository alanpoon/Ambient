use std::{path::PathBuf, sync::OnceLock};
#[cfg(target_os = "android")]
use lazy_static::lazy_static;
#[cfg(target_os = "android")]
use std::sync::{Arc,Mutex};
#[cfg(target_os = "android")]
use android_activity::AndroidApp;
lazy_static!{
    //pub static ref ANDROID_APP :Arc<Mutex<Option<AndroidApp>>> = Arc::new(Mutex::new(None));
    pub static ref DIRECTORY :Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
}
use directories::ProjectDirs;
#[cfg(target_os = "android")]
pub fn init(android:AndroidApp){
    //let path = android.internal_data_path().unwrap();
    let path = PathBuf::from("/data/user/0/dev.rustropy.wry2/files");
    *DIRECTORY.lock().unwrap() = Some(path);
    //*ANDROID_APP.lock().unwrap() = Some(android);
}
#[cfg(target_os = "ios")]
use objc::runtime::{Object, Class};
#[cfg(target_os = "ios")]
use objc::{msg_send, sel, sel_impl};
use std::ffi::CString;
use std::ffi::CStr;
#[cfg(target_os = "ios")]
pub fn init(){
    let path = unsafe {
        // Get the main bundle
        let ns_bundle: *mut Object = msg_send![Class::get("NSBundle").unwrap(), mainBundle];

        // Get the resource path as an NSString
        let ns_string: *mut Object = msg_send![ns_bundle, resourcePath];

        if ns_string.is_null() {
            return None;
        }

        // Convert NSString to a Rust string
        let c_str: *const libc::c_char = msg_send![ns_string, UTF8String];
        let path = CStr::from_ptr(c_str).to_string_lossy().into_owned();

        Some(PathBuf::from(path))
    };
    *DIRECTORY.lock().unwrap() = path;
}

pub fn dirs()->PathBuf{
    if let Some(ref a)=*DIRECTORY.lock().unwrap(){
        a.clone()
    }else{
        PathBuf::new()
    }
}

pub fn settings_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
    // use jni::objects::JObject;
    // use jni::objects::JString;
    // use std::ffi::CStr;
    // let ctx = ndk_context::android_context();
    // let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    // let context: JObject<'_> = unsafe { JObject::from_raw(ctx.context().cast()) };
    // let env = vm.attach_current_thread().unwrap();

    // let cache_dir = env.call_method(context,  "getCacheDir", "()Ljava/io/File;",&[]).unwrap().l().unwrap();

    // let path_string = env.call_method(cache_dir, "getPath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
    // let path_string = JString::from(path_string);
    // let path_chars = env.get_string_utf_chars(path_string).unwrap();

    // let rust_string = unsafe {  CStr::from_ptr(path_chars).to_str().unwrap() };
    // let cd_c = PathBuf::from(rust_string);
    // cd_c.join("config").join("settings.toml")
    // if let Some(ref android_app) = *ANDROID_APP.lock().unwrap(){
    //   //  let internal_path = android_app.internal_data_path().unwrap();
    //     internal_path.join("config").join("settings.toml")
    // }else{
    //     project_dirs().config_dir().to_owned().join("settings.toml")
    // }
    dirs().join("config").join("settings.toml")
    }
    #[cfg(not(target_os = "android"))]
    {
        project_dirs().config_dir().to_owned().join("settings.toml")
    }
}
// #[cfg(not(target_os="android"))]
// /// Returns the path to the cache for the given deployment.
// pub fn deployment_cache_path(deployment: &str) -> PathBuf {
//     project_dirs()
//         .cache_dir()
//         .join("deployments")
//         .join(deployment)
// }

#[cfg(target_os="android")]
pub fn deployment_cache_path(deployment: &str) -> PathBuf {
    // let ctx = ndk_context::android_context();
    // let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    // let context = unsafe { JObject::from_raw(ctx.context().cast()) };
    // let env = vm.attach_current_thread().unwrap();

    // // Query the global Audio Service
    // //let context_class = env.find_class("android/content/Context").expect("Class not found");
    // let cache_dir = env.call_method(context, "getCacheDir", "()Ljava/io/File;",&[]).expect("Method call failed");
    // let cache_dir_path:String = env.get_string(cache_dir.l().unwrap().into()).expect("String conversion failed").into();

    // let cd_c = PathBuf::from(cache_dir_path);
    // cd_c.join("cache").join("deployments").join(deployment)
    //PathBuf::from("")
    dirs()
}
#[cfg(not(target_os="android"))]
/// Returns the path to the cache for the given deployment.
pub fn deployment_cache_path(deployment: &str) -> PathBuf {
    project_dirs()
        .cache_dir()
        .join("deployments")
        .join(deployment)
}
fn project_dirs() -> &'static ProjectDirs {
    const QUALIFIER: &str = "com";
    const ORGANIZATION: &str = "Ambient";
    const APPLICATION: &str = "Ambient";

    static CELL: OnceLock<ProjectDirs> = OnceLock::new();
    CELL.get_or_init(|| {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .expect("failed to open project directory for Ambient")
    })
}
