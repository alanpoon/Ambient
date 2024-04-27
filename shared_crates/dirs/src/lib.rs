use std::{path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;
pub fn settings_path() -> PathBuf {
    #[cfg(target_os = "android")]
    {
    use jni::objects::JObject;
    use jni::objects::JString;
    use std::ffi::CStr;
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    let context: JObject<'_> = unsafe { JObject::from_raw(ctx.context().cast()) };
    let env = vm.attach_current_thread().unwrap();

    let cache_dir = env.call_method(context,  "getCacheDir", "()Ljava/io/File;",&[]).unwrap().l().unwrap();

    let path_string = env.call_method(cache_dir, "getPath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap();
    let path_string = JString::from(path_string);
    let path_chars = env.get_string_utf_chars(path_string).unwrap();

    let rust_string = unsafe {  CStr::from_ptr(path_chars).to_str().unwrap() };
    let cd_c = PathBuf::from(rust_string);
    cd_c.join("config").join("settings.toml")
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
    log::debug!("deployment_cache_path");
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
    PathBuf::from("")
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
