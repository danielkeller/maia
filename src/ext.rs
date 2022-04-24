use crate::ffi::Str;

pub const SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_surface\0") };
pub const WIN32_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_win32_surface\0") };
pub const WAYLAND_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_wayland_surface\0") };
pub const XLIB_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_xlib_surface\0") };
pub const XCB_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_xcb_surface\0") };
pub const ANDROID_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_android_surface\0") };
pub const METAL_SURFACE: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_EXT_metal_surface\0") };
pub const GET_PHYSICAL_DEVICE_PROPERTIES2: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_get_physical_device_properties2\0") };

pub const PORTABILITY_SUBSET: Str<'static> =
    unsafe { Str::new_unchecked(b"VK_KHR_portability_subset\0") };
