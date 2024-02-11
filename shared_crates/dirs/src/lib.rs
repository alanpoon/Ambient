use std::{path::PathBuf, sync::OnceLock};

use directories::ProjectDirs;
// #[cfg(target_os="android")]
use jni::objects::JObject;
//#[cfg(not(target_os="android"))]

/// Returns the path to the settings file.
// pub fn settings_path() -> PathBuf {
//     project_dirs().config_dir().to_owned().join("settings.toml")
// }
// #[cfg(target_os="android")]
pub fn settings_path() -> PathBuf {
    // let ctx = ndk_context::android_context();
    // let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
    // let context: JObject<'_> = unsafe { JObject::from_raw(ctx.context().cast()) };
    // let env = vm.attach_current_thread().unwrap();
    //
    // jclass activityClass = env->FindClass( "android/app/NativeActivity" );
    // jmethodID getCacheDir = env->GetMethodID( activityClass, "getCacheDir", "()Ljava/io/File;" );
    // jobject cache_dir = env->CallObjectMethod( app->activity->clazz, getCacheDir );

    // jclass fileClass = env->FindClass( "java/io/File" );
    // jmethodID getPath = env->GetMethodID( fileClass, "getPath", "()Ljava/lang/String;" );
    // jstring path_string = (jstring)env->CallObjectMethod( cache_dir, getPath );

    // const char *path_chars = env->GetStringUTFChars( path_string, NULL );
    // std::string temp_folder( path_chars );

    // env->ReleaseStringUTFChars( path_string, path_chars );
    // app->activity->vm->DetachCurrentThread();
    // // Query the global Audio Service
    //let context_class = env.find_class("android/app/NativeActivity").expect("Class not found");
    // let getCacheDir = env.get_method_id(context_class, "getCacheDir", "()Ljava/io/File;").expect("Method call failed");
    // let cache_dir = env.call_method(context, "getCacheDir", "()Ljava/io/File;",&[]).expect("Method call failed");
    // let cache_dir_path:String = env.get_string(cache_dir.l().unwrap().into()).expect("String conversion failed").into();

    //let cache_dir:String = env.call_method(getCacheDir.l().unwrap().into()).expect("String conversion failed").into();
    // Call getAbsolutePath method on the File object
    // let file_class = env.find_class("java/io/File").expect("Class not found");
    // let absolute_path = env.call_method(cache_dir, "getAbsolutePath", "()Ljava/lang/String;",&[]).expect("Method call failed");

    // // Convert the Java String to Rust String
    // let absolute_path_str: String = env.get_string(absolute_path.into()).expect("String conversion failed").into();
    let cache_dir_path = String::from("");
    let cd_c = PathBuf::from(cache_dir_path);
    //let cd_c = PathBuf::from("absolute_path_str");
    cd_c.join("config").join("settings.toml")
}
// #[cfg(not(target_os="android"))]
// /// Returns the path to the cache for the given deployment.
// pub fn deployment_cache_path(deployment: &str) -> PathBuf {
//     project_dirs()
//         .cache_dir()
//         .join("deployments")
//         .join(deployment)
// }

// #[cfg(target_os="android")]
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
