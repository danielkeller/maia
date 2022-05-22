use crate::enums::Bool;
use crate::enums::*;
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
    Option<ArrayMut<MaybeUninit<ExtensionProperties>>>,
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
        Option<ArrayMut<MaybeUninit<Handle<VkPhysicalDevice>>>>,
    ) -> VkResult,
    pub get_physical_device_properties: unsafe extern "system" fn(
        Ref<VkPhysicalDevice>,
        &mut MaybeUninit<PhysicalDeviceProperties>,
    ),
    pub get_physical_device_queue_family_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            &mut u32,
            Option<ArrayMut<MaybeUninit<QueueFamilyProperties>>>,
        ),
    pub enumerate_device_extension_properties:
        unsafe extern "system" fn(
            Ref<VkPhysicalDevice>,
            Option<Str<'_>>,
            &mut u32,
            Option<ArrayMut<MaybeUninit<ExtensionProperties>>>,
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
        &mut Option<Handle<VkQueue>>,
    ),
    pub queue_submit: unsafe extern "system" fn(
        Mut<VkQueue>,
        u32,
        Option<Array<VkSubmitInfo<Null>>>,
        Option<Mut<VkFence>>,
    ) -> VkResult,
    pub queue_wait_idle: unsafe extern "system" fn(Mut<VkQueue>) -> VkResult,
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
        Array<Ref<VkFence>>,
        Bool,
        u64,
    ) -> VkResult,
    pub reset_fences: unsafe extern "system" fn(
        Ref<VkDevice>,
        u32,
        Array<Mut<VkFence>>,
    ) -> VkResult,
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
    pub create_image_view: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkImageViewCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkImageView>>,
    ) -> VkResult,
    pub destroy_image_view: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkImageView>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_framebuffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        &VkFramebufferCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkFramebuffer>>,
    ) -> VkResult,
    pub destroy_framebuffer: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkFramebuffer>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_render_pass: unsafe extern "system" fn(
        Ref<VkDevice>,
        &RenderPassCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkRenderPass>>,
    ) -> VkResult,
    pub destroy_render_pass: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkRenderPass>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub create_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        &CommandPoolCreateInfo,
        Option<&'_ AllocationCallbacks>,
        &mut Option<Handle<VkCommandPool>>,
    ) -> VkResult,
    pub destroy_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        Option<&'_ AllocationCallbacks>,
    ),
    pub reset_command_pool: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        CommandPoolResetFlags,
    ) -> VkResult,
    pub allocate_command_buffers: unsafe extern "system" fn(
        Ref<VkDevice>,
        &CommandBufferAllocateInfo<'_>,
        ArrayMut<MaybeUninit<Handle<VkCommandBuffer>>>,
    ) -> VkResult,
    pub free_command_buffers: unsafe extern "system" fn(
        Ref<VkDevice>,
        Mut<VkCommandPool>,
        u32,
        &Mut<VkCommandBuffer>,
    ),
    pub begin_command_buffer: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        &CommandBufferBeginInfo,
    ) -> VkResult,
    pub end_command_buffer:
        unsafe extern "system" fn(Mut<VkCommandBuffer>) -> VkResult,
    pub cmd_clear_color_image: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        Ref<VkImage>,
        ImageLayout,
        &ClearColorValue,
        u32,
        Array<ImageSubresourceRange>,
    ),
    pub cmd_pipeline_barrier: unsafe extern "system" fn(
        Mut<VkCommandBuffer>,
        PipelineStageFlags,
        PipelineStageFlags,
        DependencyFlags,
        u32,
        Option<Array<MemoryBarrier>>,
        u32,
        Option<Array<VkBufferMemoryBarrier>>,
        u32,
        Option<Array<VkImageMemoryBarrier>>,
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
                queue_submit: transmute(inst.load(device, "vkQueueSubmit\0")),
                queue_wait_idle: transmute(
                    inst.load(device, "vkQueueWaitIdle\0"),
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
                create_image_view: transmute(
                    inst.load(device, "vkCreateImageView\0"),
                ),
                destroy_image_view: transmute(
                    inst.load(device, "vkDestroyImageView\0"),
                ),
                create_framebuffer: transmute(
                    inst.load(device, "vkCreateFramebuffer\0"),
                ),
                destroy_framebuffer: transmute(
                    inst.load(device, "vkDestroyFramebuffer\0"),
                ),
                create_render_pass: transmute(
                    inst.load(device, "vkCreateRenderPass\0"),
                ),
                destroy_render_pass: transmute(
                    inst.load(device, "vkDestroyRenderPass\0"),
                ),
                create_command_pool: transmute(
                    inst.load(device, "vkCreateCommandPool\0"),
                ),
                destroy_command_pool: transmute(
                    inst.load(device, "vkDestroyCommandPool\0"),
                ),
                reset_command_pool: transmute(
                    inst.load(device, "vkResetCommandPool\0"),
                ),
                allocate_command_buffers: transmute(
                    inst.load(device, "vkAllocateCommandBuffers\0"),
                ),
                free_command_buffers: transmute(
                    inst.load(device, "vkFreeCommandBuffers\0"),
                ),
                begin_command_buffer: transmute(
                    inst.load(device, "vkBeginCommandBuffer\0"),
                ),
                end_command_buffer: transmute(
                    inst.load(device, "vkEndCommandBuffer\0"),
                ),
                cmd_clear_color_image: transmute(
                    inst.load(device, "vkCmdClearColorImage\0"),
                ),
                cmd_pipeline_barrier: transmute(
                    inst.load(device, "vkCmdPipelineBarrier\0"),
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
