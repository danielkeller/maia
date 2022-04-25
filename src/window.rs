use std::ptr::NonNull;
use std::sync::Arc;

use crate::ext;
use crate::ext::khr_surface::SurfaceKHR;
use crate::ffi::*;
use crate::instance::Instance;
use crate::types::*;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub fn required_instance_extensions(
    window: &impl HasRawWindowHandle,
) -> Result<&'static [Str<'static>]> {
    let extensions = match window.raw_window_handle() {
        #[cfg(target_os = "windows")]
        RawWindowHandle::Win32(_) => {
            const WINDOWS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WIN32_SURFACE];
            &WINDOWS_EXTS
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        RawWindowHandle::Wayland(_) => {
            const WAYLAND_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::WAYLAND_SURFACE];
            &WAYLAND_EXTS
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        RawWindowHandle::Xlib(_) => {
            const XLIB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XLIB_SURFACE];
            &XLIB_EXTS
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        RawWindowHandle::Xcb(_) => {
            const XCB_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::XCB_SURFACE];
            &XCB_EXTS
        }

        #[cfg(any(target_os = "android"))]
        RawWindowHandle::AndroidNdk(_) => {
            const ANDROID_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::ANDROID_SURFACE];
            &ANDROID_EXTS
        }

        #[cfg(any(target_os = "macos"))]
        RawWindowHandle::AppKit(_) => {
            const MACOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            &MACOS_EXTS
        }

        #[cfg(any(target_os = "ios"))]
        RawWindowHandle::UiKit(_) => {
            const IOS_EXTS: [Str<'static>; 2] =
                [ext::SURFACE, ext::METAL_SURFACE];
            &IOS_EXTS
        }

        _ => return Err(Error::EXTENSION_NOT_PRESENT),
    };

    Ok(extensions)
}

pub fn create_surface(
    instance: &Arc<Instance>,
    window: &impl HasRawWindowHandle,
) -> Result<Arc<SurfaceKHR>> {
    match window.raw_window_handle() {
        #[cfg(any(target_os = "macos"))]
        RawWindowHandle::AppKit(handle) => {
            use raw_window_metal::{appkit, Layer};

            unsafe {
                match appkit::metal_layer_from_handle(handle) {
                    Layer::Existing(layer) | Layer::Allocated(layer) => {
                        instance.ext_metal_surface().create_metal_surface_ext(
                            &MetalSurfaceCreateInfoEXT::S {
                                next: None,
                                flags: Default::default(),
                                layer: NonNull::new(layer as *mut c_void)
                                    .unwrap(),
                            },
                        )
                    }
                    Layer::None => Err(Error::INITIALIZATION_FAILED),
                }
            }
        }
        _ => Err(Error::EXTENSION_NOT_PRESENT),
    }
}
