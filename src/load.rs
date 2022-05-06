use crate::ffi::*;
use crate::instance::Instance;
use crate::types::*;
use std::ffi::c_void;
use std::mem::transmute;
use std::mem::MaybeUninit;

extern "system" {
    fn vkGetInstanceProcAddr(
        instance: Option<InstanceRef<'_>>,
        name: Str<'_>,
    ) -> Option<NonNull<c_void>>;
}

pub unsafe fn vk_create_instance() -> unsafe extern "system" fn(
    &'_ InstanceCreateInfo<'_>,
    Option<&'_ AllocationCallbacks>,
    &mut Option<InstanceRef<'static>>,
) -> VkResult {
    transmute(load(None, "vkCreateInstance\0"))
}

pub unsafe fn vk_enumerate_instance_extension_properties(
) -> unsafe extern "system" fn(
    Option<Str<'_>>,
    &mut u32,
    Option<&mut MaybeUninit<ExtensionProperties>>,
) -> VkResult {
    transmute(load(None, "vkEnumerateInstanceExtensionProperties\0"))
}

pub struct InstanceFn {
    pub destroy_instance: unsafe extern "system" fn(
        InstanceRef<'static>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub enumerate_physical_devices: unsafe extern "system" fn(
        InstanceRef<'_>,
        &mut u32,
        Option<&mut MaybeUninit<PhysicalDeviceRef<'_>>>,
    ) -> VkResult,
    pub get_physical_device_properties: unsafe extern "system" fn(
        PhysicalDeviceRef<'_>,
        &mut MaybeUninit<PhysicalDeviceProperties>,
    ),
    pub get_physical_device_queue_family_properties:
        unsafe extern "system" fn(
            PhysicalDeviceRef<'_>,
            &mut u32,
            Option<&mut MaybeUninit<QueueFamilyProperties>>,
        ),
    pub enumerate_device_extension_properties:
        unsafe extern "system" fn(
            PhysicalDeviceRef<'_>,
            Option<Str<'_>>,
            &mut u32,
            Option<&mut MaybeUninit<ExtensionProperties>>,
        ) -> VkResult,
    pub create_device: unsafe extern "system" fn(
        PhysicalDeviceRef<'_>,
        &'_ DeviceCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<DeviceRef<'static>>,
    ) -> VkResult,
    pub get_device_proc_addr:
        unsafe extern "system" fn(
            DeviceRef<'_>,
            name: Str<'_>,
        ) -> Option<NonNull<c_void>>,
}

impl InstanceFn {
    pub fn new(inst: InstanceRef<'_>) -> Self {
        let inst = Some(inst);
        unsafe {
            Self {
                destroy_instance: transmute(load(inst, "vkDestroyInstance\0")),
                create_device: transmute(load(inst, "vkCreateDevice\0")),
                get_physical_device_properties: transmute(load(
                    inst,
                    "vkGetPhysicalDeviceProperties\0",
                )),
                get_physical_device_queue_family_properties: transmute(load(
                    inst,
                    "vkGetPhysicalDeviceQueueFamilyProperties\0",
                )),
                enumerate_device_extension_properties: transmute(load(
                    inst,
                    "vkEnumerateDeviceExtensionProperties\0",
                )),
                get_device_proc_addr: transmute(load(
                    inst,
                    "vkGetDeviceProcAddr\0",
                )),
                enumerate_physical_devices: transmute(load(
                    inst,
                    "vkEnumeratePhysicalDevices\0",
                )),
            }
        }
    }
}

/// Load instance function. Panics if the string is not null-terminated or the
/// function was not found.
pub fn load(instance: Option<InstanceRef<'_>>, name: &str) -> NonNull<c_void> {
    let ptr =
        unsafe { vkGetInstanceProcAddr(instance, name.try_into().unwrap()) };
    ptr.unwrap_or_else(|| {
        panic!("Could not load {:?}", &name[0..name.len() - 1])
    })
}

pub struct DeviceFn {
    pub destroy_device: unsafe extern "system" fn(
        DeviceRef<'static>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_device_queue: unsafe extern "system" fn(
        DeviceRef<'_>,
        u32,
        u32,
        &mut Option<QueueRef<'_>>,
    ),
}

impl DeviceFn {
    pub fn new(inst: &Instance, device: DeviceRef<'_>) -> Self {
        unsafe {
            Self {
                destroy_device: {
                    transmute(inst.load(device, "vkDestroyDevice\0"))
                },
                get_device_queue: transmute(
                    inst.load(device, "vkGetDeviceQueue\0"),
                ),
            }
        }
    }
}

impl Instance {
    /// Load device function. Panics if the string is not null-terminated or the
    /// function was not found.
    pub(crate) fn load(
        &self,
        device: DeviceRef<'_>,
        name: &str,
    ) -> NonNull<c_void> {
        let ptr = unsafe {
            (self.fun.get_device_proc_addr)(device, name.try_into().unwrap())
        };
        ptr.unwrap_or_else(|| {
            panic!("Could not load {:?}", &name[0..name.len() - 1])
        })
    }
}
