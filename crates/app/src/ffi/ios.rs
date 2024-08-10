//use core_graphics::{base::CGFloat, geometry::CGRect};
use libc::c_void;
use objc::{runtime::Object, *};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::marker::Sync;
use std::sync::Arc;

#[repr(C)]
pub struct IOSViewObj {
    // metal_layer 所在的 UIView 容器
    // UIView 有一系列方便的函数可供我们在 Rust 端来调用
    pub view: *mut Object,
    // 指向 iOS 端 CAMetalLayer 的指针
    pub metal_layer: *mut c_void,
    // 不同的 iOS 设备支持不同的屏幕刷新率，有时我们的 GPU 程序需要用到这类信息
    pub maximum_frames: i32,
    // 外部函数接口，用于给 iOS 端传递状态码
    pub callback_to_swift: extern "C" fn(arg: i32),
}
// unsafe impl HasRawWindowHandle for IOSViewObj {
//     fn raw_window_handle(&self) -> RawWindowHandle {
//         // Use the appropriate RawWindowHandle variant for your platform
//         // Here, we use a dummy implementation for demonstration
//         RawWindowHandle::Win32(raw_window_handle::Win32Handle {
//             hwnd: self.metal_layer as *mut _,
//             hinstance: std::ptr::null_mut(),
//             ..raw_window_handle::Win32Handle::empty()
//         })
//     }
// }
#[no_mangle]
pub fn create_wgpu_canvas(ios_obj: IOSViewObj) -> *mut libc::c_void {
    println!(
        "create_wgpu_canvas, maximum frames: {}",
        ios_obj.maximum_frames
    );
    let obj = ios_obj;
    //let obj = WgpuCanvas::new(AppSurface::new(ios_obj), 0_i32);
    // 使用 Box 对 Rust 对象进行装箱操作。
    // 我们无法将 Rust 对象直接传递给外部语言，通过装箱来传递此对象的胖指针
    let box_obj = Box::new(obj);
    // into_raw 返回指针的同时，将此对象的内存管理权转交给调用方
    Box::into_raw(box_obj) as *mut libc::c_void
}