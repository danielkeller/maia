use std::mem::{transmute, MaybeUninit};

use crate::device::Device;
use crate::enums::*;
use crate::ffi::ArrayMut;
use crate::instance::Instance;
use crate::types::*;

pub struct SurfaceKHRFn {
    pub destroy_surface_khr: unsafe extern "system" fn(
        Ref<VkInstance>,
        Mut<VkSurfaceKHR>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_physical_device_surface_support_khr:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            u32,
            Ref<VkSurfaceKHR>,
            &mut Bool,
        ) -> VkResult,
    pub get_physical_device_surface_capabilities_khr:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            Ref<VkSurfaceKHR>,
            &mut MaybeUninit<SurfaceCapabilitiesKHR>,
        ) -> VkResult,
    pub get_physical_device_surface_formats_khr:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            Ref<VkSurfaceKHR>,
            &mut u32,
            Option<ArrayMut<MaybeUninit<SurfaceFormatKHR>>>,
        ) -> VkResult,
}

impl SurfaceKHRFn {
    pub fn new(inst: &Instance) -> Self {
        unsafe {
            Self {
                destroy_surface_khr: transmute(
                    inst.get_proc_addr("vkDestroySurfaceKHR\0"),
                ),
                get_physical_device_surface_support_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceSupportKHR\0",
                    ),
                ),
                get_physical_device_surface_capabilities_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceCapabilitiesKHR\0",
                    ),
                ),
                get_physical_device_surface_formats_khr: transmute(
                    inst.get_proc_addr(
                        "vkGetPhysicalDeviceSurfaceFormatsKHR\0",
                    ),
                ),
            }
        }
    }
}

pub struct MetalSurfaceFn {
    pub create_metal_surface_ext: unsafe extern "system" fn(
        Ref<VkInstance>,
        &MetalSurfaceCreateInfoEXT,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSurfaceKHR>>,
    ) -> VkResult,
}

impl MetalSurfaceFn {
    pub fn new(inst: &Instance) -> Self {
        unsafe {
            Self {
                create_metal_surface_ext: transmute(
                    inst.get_proc_addr("vkCreateMetalSurfaceEXT\0"),
                ),
            }
        }
    }
}

pub struct SwapchainDeviceFn {
    pub create_swapchain_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkSwapchainCreateInfoKHR,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSwapchainKHR>>,
    ) -> VkResult,
}

impl SwapchainDeviceFn {
    pub fn new(dev: &Device) -> Self {
        unsafe {
            Self {
                create_swapchain_khr: transmute(
                    dev.get_proc_addr("vkCreateSwapchainKHR\0"),
                ),
            }
        }
    }
}

pub struct SwapchainKHRFn {
    pub destroy_swapchain_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSwapchainKHR>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_swapchain_images_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Ref<VkSwapchainKHR>,
        &mut u32,
        Option<ArrayMut<MaybeUninit<Handle<VkImage>>>>,
    ) -> VkResult,
    pub acquire_next_image_khr: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSwapchainKHR>,
        u64,
        Option<Mut<VkSemaphore>>,
        Option<Mut<VkFence>>,
        &mut u32,
    ) -> VkResult,
    pub queue_present_khr: unsafe extern "system" fn(
        Mut<VkQueue>,
        &PresentInfoKHR<'_>,
    ) -> VkResult,
}

impl SwapchainKHRFn {
    pub fn new(dev: &Device) -> Self {
        unsafe {
            Self {
                destroy_swapchain_khr: transmute(
                    dev.get_proc_addr("vkDestroySwapchainKHR\0"),
                ),
                get_swapchain_images_khr: transmute(
                    dev.get_proc_addr("vkGetSwapchainImagesKHR\0"),
                ),
                acquire_next_image_khr: transmute(
                    dev.get_proc_addr("vkAcquireNextImageKHR\0"),
                ),
                queue_present_khr: transmute(
                    dev.get_proc_addr("vkQueuePresentKHR\0"),
                ),
            }
        }
    }
}
