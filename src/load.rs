use crate::enums::Bool;
use crate::ffi::*;
use crate::instance::Instance;
use crate::types::*;
use std::ffi::c_void;
use std::mem::transmute;
use std::mem::MaybeUninit;

extern "system" {
    fn vkGetInstanceProcAddr(
        instance: Option<Ref<VkInstance>>,
        name: Str<'_>,
    ) -> Option<NonNull<c_void>>;
}

pub unsafe fn vk_create_instance() -> unsafe extern "system" fn(
    &'_ InstanceCreateInfo<'_>,
    Option<&'_ AllocationCallbacks>,
    &mut Option<Handle<VkInstance>>,
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
        Mut<VkInstance>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub enumerate_physical_devices: unsafe extern "system" fn(
        Ref<VkInstance>,
        &mut u32,
        Option<&mut MaybeUninit<Ref<VkPhysicalDevice>>>,
    ) -> VkResult,
    pub get_physical_device_properties: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &mut MaybeUninit<PhysicalDeviceProperties>,
    ),
    pub get_physical_device_queue_family_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            &mut u32,
            Option<&mut MaybeUninit<QueueFamilyProperties>>,
        ),
    pub enumerate_device_extension_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            Option<Str<'_>>,
            &mut u32,
            Option<&mut MaybeUninit<ExtensionProperties>>,
        ) -> VkResult,
    pub create_device: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &'_ DeviceCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkDevice>>,
    ) -> VkResult,
    pub get_device_proc_addr:
        unsafe extern "system" fn(
            Ref<VkDevice>,
            name: Str<'_>,
        ) -> Option<NonNull<c_void>>,
}

impl InstanceFn {
    pub fn new(inst: Ref<VkInstance>) -> Self {
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
pub fn load(instance: Option<Ref<VkInstance>>, name: &str) -> NonNull<c_void> {
    let ptr =
        unsafe { vkGetInstanceProcAddr(instance, name.try_into().unwrap()) };
    ptr.unwrap_or_else(|| {
        panic!("Could not load {:?}", &name[0..name.len() - 1])
    })
}

pub struct DeviceFn {
    pub destroy_device: unsafe extern "system" fn(
        Mut<VkDevice>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub get_device_queue: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        u32,
        &mut Option<Ref<VkQueue>>,
    ),
    pub create_fence: unsafe extern "system" fn(
        Ref<VkDevice>,
        &FenceCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkFence>>,
    ) -> VkResult,
    pub destroy_fence: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkFence>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub wait_for_fences: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        &Ref<VkFence>,
        Bool,
        u64,
    ) -> VkResult,
    pub reset_fences:
        unsafe extern "system" fn(Ref<VkDevice>, u32, Mut<VkFence>) -> VkResult,
    pub create_semaphore: unsafe extern "system" fn(
        Ref<VkDevice>,
        &SemaphoreCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkSemaphore>>,
    ) -> VkResult,
    pub destroy_semaphore: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkSemaphore>,
        Option<&'_ AllocationCallbacks>,
    ),
}

impl DeviceFn {
    pub fn new(inst: &Instance, device: Ref<VkDevice>) -> Self {
        unsafe {
            Self {
                destroy_device: {
                    transmute(inst.load(device, "vkDestroyDevice\0"))
                },
                get_device_queue: transmute(
                    inst.load(device, "vkGetDeviceQueue\0"),
                ),
                create_fence: transmute(inst.load(device, "vkCreateFence\0")),
                destroy_fence: transmute(inst.load(device, "vkDestroyFence\0")),
                wait_for_fences: transmute(
                    inst.load(device, "vkWaitForFences\0"),
                ),
                reset_fences: transmute(inst.load(device, "vkResetFences\0")),
                create_semaphore: transmute(
                    inst.load(device, "vkCreateSemaphore\0"),
                ),
                destroy_semaphore: transmute(
                    inst.load(device, "vkDestroySemaphore\0"),
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
        device: Ref<VkDevice>,
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
