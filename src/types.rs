use crate::enums::*;
use crate::ffi::*;
use std::fmt::Debug;
pub use std::sync::Arc;

use std::num::NonZeroI32;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct VkError(pub NonZeroI32);
const fn _err(code: i32) -> VkError {
    match NonZeroI32::new(code) {
        Some(i) => VkError(i),
        None => panic!("VkError code cannot be 0"),
    }
}

impl std::fmt::Display for VkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}
impl std::error::Error for VkError {}

pub type VkResult = std::result::Result<(), VkError>;

// Check that VkResult corresponds to Vulkan's definition. This allows wrapper
// functions to use '?'.
const _: () = assert!(std::mem::size_of::<VkResult>() == 4);
const _: () =
    assert!(unsafe { std::mem::transmute::<i32, VkResult>(0).is_ok() });
const _EXPECTED: VkResult = Err(_err(-1));
const _: () = assert!(matches!(
    unsafe { std::mem::transmute::<i32, VkResult>(-1) },
    _EXPECTED
));

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NonNullDispatchableHandle(NonNull<c_void>);
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NonNullNonDispatchableHandle(std::num::NonZeroU64);

impl Debug for NonNullDispatchableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl Debug for NonNullNonDispatchableHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

// This hides its pointer and is thus thread safe.
unsafe impl Send for NonNullDispatchableHandle {}
unsafe impl Sync for NonNullDispatchableHandle {}

/// Owned Vulkan handle
#[repr(transparent)]
pub struct Handle<T> {
    _value: T,
}
impl<T: Debug> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}
impl<T: Copy> Handle<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        Ref { _value: self._value, _lt: PhantomData }
    }
    pub fn borrow_mut(&mut self) -> Mut<'_, T> {
        Mut { _value: self._value, _lt: PhantomData }
    }
    /// It is not UB to clone this type, the clones just can't be used at the
    /// same time.
    pub unsafe fn clone(&self) -> Self {
        Self { ..*self }
    }
}

/// Borrowed Vulkan handle
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Ref<'a, T> {
    _value: T,
    _lt: PhantomData<&'a T>,
}
impl<T: Debug> Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}

/// Mutably borrowed Vulkan handle
#[repr(transparent)]
pub struct Mut<'a, T> {
    _value: T,
    _lt: PhantomData<&'a T>,
}
impl<T: Debug> Debug for Mut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._value.fmt(f)
    }
}
impl<'a, T: Copy> Mut<'a, T> {
    // Note that this cannot be used to extend 'a, since &'b Self<'a>
    // requires 'a: 'b
    pub fn reborrow(&self) -> Ref<'_, T> {
        Ref { _value: self._value, _lt: PhantomData }
    }
    pub fn reborrow_mut(&mut self) -> Mut<'_, T> {
        Self { ..*self }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkInstance(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkPhysicalDevice(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkDevice(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkQueue(NonNullDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkSemaphore(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkFence(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkImage(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkFramebuffer(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkCommandPool(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkCommandBuffer(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkSurfaceKHR(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VkSwapchainKHR(NonNullNonDispatchableHandle);

/// u32 with only one allowed value
macro_rules! structure_type {
    ($name: ident, $value: literal) => {
        #[repr(u32)]
        #[derive(Debug)]
        pub enum $name {
            Value = $value,
        }
        impl Default for $name {
            fn default() -> Self {
                Self::Value
            }
        }
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Extent2D {
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ClearColorValue {
    f: [f32; 4],
    i: [i32; 4],
    u: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImageSubresourceRange {
    aspect_mask: ImageAspectFlags,
    base_mip_level: u32,
    level_count: u32,
    base_array_layer: u32,
    layer_count: u32,
}

impl Default for ImageSubresourceRange {
    /// The entirety of a color image
    fn default() -> Self {
        Self {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: u32::MAX,
            base_array_layer: 0,
            layer_count: u32::MAX,
        }
    }
}

pub enum AllocationCallbacks {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct InstanceCreateInfo<'a, Next = Null, AINext = Null> {
    pub stype: InstanceCreateInfoType,
    pub next: Next,
    pub flags: InstanceCreateFlags,
    pub application_info: Option<&'a ApplicationInfo<'a, AINext>>,
    pub enabled_layer_names: Slice<'a, Str<'a>>,
    pub enabled_extension_names: Slice<'a, Str<'a>>,
}
structure_type!(InstanceCreateInfoType, 1);

#[repr(C)]
#[derive(Debug, Default)]
pub struct ApplicationInfo<'a, Next> {
    pub stype: ApplicationInfoType,
    pub next: Next,
    pub application_name: Str<'a>,
    pub application_version: u32,
    pub engine_name: Str<'a>,
    pub engine_version: u32,
    pub api_version: u32,
}
structure_type!(ApplicationInfoType, 0);

#[repr(C)]
#[derive(Debug)]
pub struct ExtensionProperties {
    pub extension_name: CharArray<MAX_EXTENSION_NAME_SIZE>,
    pub spec_version: u32,
}

pub const MAX_EXTENSION_NAME_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug)]
pub struct PhysicalDeviceProperties {
    pub api_version: u32,
    pub driver_version: u32,
    pub vendor_id: u32,
    pub device_id: u32,
    pub device_type: PhysicalDeviceType,
    pub device_name: CharArray<MAX_PHYSICAL_DEVICE_NAME_SIZE>,
    pub pipeline_cache_uuid: UUID,
    pub limits: PhysicalDeviceLimits,
    pub sparse_properties: PhysicalDeviceSparseProperties,
}

pub const MAX_PHYSICAL_DEVICE_NAME_SIZE: usize = 256;

#[repr(C)]
#[derive(Debug)]
pub struct PhysicalDeviceLimits {
    pub max_image_dimension_1d: u32,
    pub max_image_dimension_2d: u32,
    pub max_image_dimension_3d: u32,
    pub max_image_dimension_cube: u32,
    pub max_image_array_layers: u32,
    pub max_texel_buffer_elements: u32,
    pub max_uniform_buffer_range: u32,
    pub max_storage_buffer_range: u32,
    pub max_push_constants_size: u32,
    pub max_memory_allocation_count: u32,
    pub max_sampler_allocation_count: u32,
    pub buffer_image_granularity: u64,
    pub sparse_address_space_size: u64,
    pub max_bound_descriptor_sets: u32,
    pub max_per_stage_descriptor_samplers: u32,
    pub max_per_stage_descriptor_uniform_buffers: u32,
    pub max_per_stage_descriptor_storage_buffers: u32,
    pub max_per_stage_descriptor_sampled_images: u32,
    pub max_per_stage_descriptor_storage_images: u32,
    pub max_per_stage_descriptor_input_attachments: u32,
    pub max_per_stage_resources: u32,
    pub max_descriptor_set_samplers: u32,
    pub max_descriptor_set_uniform_buffers: u32,
    pub max_descriptor_set_uniform_buffers_dynamic: u32,
    pub max_descriptor_set_storage_buffers: u32,
    pub max_descriptor_set_storage_buffers_dynamic: u32,
    pub max_descriptor_set_sampled_images: u32,
    pub max_descriptor_set_storage_images: u32,
    pub max_descriptor_set_input_attachments: u32,
    pub max_vertex_input_attributes: u32,
    pub max_vertex_input_bindings: u32,
    pub max_vertex_input_attribute_offset: u32,
    pub max_vertex_input_binding_stride: u32,
    pub max_vertex_output_components: u32,
    pub max_tessellation_generation_level: u32,
    pub max_tessellation_patch_size: u32,
    pub max_tessellation_control_per_vertex_input_components: u32,
    pub max_tessellation_control_per_vertex_output_components: u32,
    pub max_tessellation_control_per_patch_output_components: u32,
    pub max_tessellation_control_total_output_components: u32,
    pub max_tessellation_evaluation_input_components: u32,
    pub max_tessellation_evaluation_output_components: u32,
    pub max_geometry_shader_invocations: u32,
    pub max_geometry_input_components: u32,
    pub max_geometry_output_components: u32,
    pub max_geometry_output_vertices: u32,
    pub max_geometry_total_output_components: u32,
    pub max_fragment_input_components: u32,
    pub max_fragment_output_attachments: u32,
    pub max_fragment_dual_src_attachments: u32,
    pub max_fragment_combined_output_resources: u32,
    pub max_compute_shared_memory_size: u32,
    pub max_compute_work_group_count: [u32; 3],
    pub max_compute_work_group_invocations: u32,
    pub max_compute_work_group_size: [u32; 3],
    pub sub_pixel_precision_bits: u32,
    pub sub_texel_precision_bits: u32,
    pub mipmap_precision_bits: u32,
    pub max_draw_indexed_index_value: u32,
    pub max_draw_indirect_count: u32,
    pub max_sampler_lod_bias: f32,
    pub max_sampler_anisotropy: f32,
    pub max_viewports: u32,
    pub max_viewport_dimensions: [u32; 2],
    pub viewport_bounds_range: [f32; 2],
    pub viewport_sub_pixel_bits: u32,
    pub min_memory_map_alignment: usize,
    pub min_texel_buffer_offset_alignment: u64,
    pub min_uniform_buffer_offset_alignment: u64,
    pub min_storage_buffer_offset_alignment: u64,
    pub min_texel_offset: i32,
    pub max_texel_offset: u32,
    pub min_texel_gather_offset: i32,
    pub max_texel_gather_offset: u32,
    pub min_interpolation_offset: f32,
    pub max_interpolation_offset: f32,
    pub sub_pixel_interpolation_offset_bits: u32,
    pub max_framebuffer_width: u32,
    pub max_framebuffer_height: u32,
    pub max_framebuffer_layers: u32,
    pub framebuffer_color_sample_counts: SampleCountFlags,
    pub framebuffer_depth_sample_counts: SampleCountFlags,
    pub framebuffer_stencil_sample_counts: SampleCountFlags,
    pub framebuffer_no_attachments_sample_counts: SampleCountFlags,
    pub max_color_attachments: u32,
    pub sampled_image_color_sample_counts: SampleCountFlags,
    pub sampled_image_integer_sample_counts: SampleCountFlags,
    pub sampled_image_depth_sample_counts: SampleCountFlags,
    pub sampled_image_stencil_sample_counts: SampleCountFlags,
    pub storage_image_sample_counts: SampleCountFlags,
    pub max_sample_mask_words: u32,
    pub timestamp_compute_and_graphics: Bool,
    pub timestamp_period: f32,
    pub max_clip_distances: u32,
    pub max_cull_distances: u32,
    pub max_combined_clip_and_cull_distances: u32,
    pub discrete_queue_priorities: u32,
    pub point_size_range: [f32; 2],
    pub line_width_range: [f32; 2],
    pub point_size_granularity: f32,
    pub line_width_granularity: f32,
    pub strict_lines: Bool,
    pub standard_sample_locations: Bool,
    pub optimal_buffer_copy_offset_alignment: u64,
    pub optimal_buffer_copy_row_pitch_alignment: u64,
    pub non_coherent_atom_size: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct PhysicalDeviceSparseProperties {
    pub residency_standard_2d_block_shape: Bool,
    pub residency_standard_2d_multisample_block_shape: Bool,
    pub residency_standard_3d_block_shape: Bool,
    pub residency_aligned_mip_size: Bool,
    pub residency_non_resident_strict: Bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct QueueFamilyProperties {
    pub queue_flags: QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: Extent3D,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DeviceCreateInfo<'a, Next = Null> {
    pub stype: DeviceCreateInfoType,
    pub next: Next,
    pub flags: DeviceCreateFlags,
    pub queue_create_infos: Slice_<'a, DeviceQueueCreateInfo<'a>>,
    pub enabled_layer_names: Slice<'a, Str<'a>>,
    pub enabled_extension_names: Slice<'a, Str<'a>>,
    pub enabled_features: Option<&'a PhysicalDeviceFeatures>,
}
structure_type!(DeviceCreateInfoType, 3);

pub enum DeviceCreateInfoExtension {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DeviceQueueCreateInfo<'a, Next = Null> {
    pub stype: DeviceQueueCreateInfoType,
    pub next: Next,
    pub flags: DeviceQueueCreateFlags,
    pub queue_family_index: u32,
    pub queue_priorities: Slice<'a, f32>,
}
structure_type!(DeviceQueueCreateInfoType, 2);

#[repr(C)]
#[derive(Default, Debug)]
pub struct PhysicalDeviceFeatures {
    robust_buffer_access: Bool,
    full_draw_index_uint32: Bool,
    image_cube_array: Bool,
    independent_blend: Bool,
    geometry_shader: Bool,
    tessellation_shader: Bool,
    sample_rate_shading: Bool,
    dual_src_blend: Bool,
    logic_op: Bool,
    multi_draw_indirect: Bool,
    draw_indirect_first_instance: Bool,
    depth_clamp: Bool,
    depth_bias_clamp: Bool,
    fill_mode_non_solid: Bool,
    depth_bounds: Bool,
    wide_lines: Bool,
    large_points: Bool,
    alpha_to_one: Bool,
    multi_viewport: Bool,
    sampler_anisotropy: Bool,
    texture_compression_etc2: Bool,
    texture_compression_astc_ldr: Bool,
    texture_compression_bc: Bool,
    occlusion_query_precise: Bool,
    pipeline_statistics_query: Bool,
    vertex_pipeline_stores_and_atomics: Bool,
    fragment_stores_and_atomics: Bool,
    shader_tessellation_and_geometry_point_size: Bool,
    shader_image_gather_extended: Bool,
    shader_storage_image_extended_formats: Bool,
    shader_storage_image_multisample: Bool,
    shader_storage_image_read_without_format: Bool,
    shader_storage_image_write_without_format: Bool,
    shader_uniform_buffer_array_dynamic_indexing: Bool,
    shader_sampled_image_array_dynamic_indexing: Bool,
    shader_storage_buffer_array_dynamic_indexing: Bool,
    shader_storage_image_array_dynamic_indexing: Bool,
    shader_clip_distance: Bool,
    shader_cull_distance: Bool,
    shader_float64: Bool,
    shader_int64: Bool,
    shader_int16: Bool,
    shader_resource_residency: Bool,
    shader_resource_min_lod: Bool,
    sparse_binding: Bool,
    sparse_residency_buffer: Bool,
    sparse_residency_image_2d: Bool,
    sparse_residency_image_3d: Bool,
    sparse_residency2_samples: Bool,
    sparse_residency4_samples: Bool,
    sparse_residency8_samples: Bool,
    sparse_residency16_samples: Bool,
    sparse_residency_aliased: Bool,
    variable_multisample_rate: Bool,
    inherited_queries: Bool,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct VkSubmitInfo<'a, Next = Null> {
    pub stype: SubmitInfoType,
    pub next: Next,
    pub wait_semaphores: Slice<'a, Ref<'a, VkSemaphore>>,
    // Safety: Must be same length as wait_semaphores
    pub wait_stage_masks: Option<Array<'a, PipelineStageFlags>>,
    pub command_buffers: Slice<'a, Mut<'a, VkCommandBuffer>>,
    pub signal_semaphores: Slice<'a, Ref<'a, VkSemaphore>>,
}
structure_type!(SubmitInfoType, 4);

#[repr(C)]
#[derive(Debug, Default)]
pub struct FenceCreateInfo<Next = Null> {
    pub stype: FenceCreateInfoType,
    pub next: Next,
    pub flags: FenceCreateFlags,
}
structure_type!(FenceCreateInfoType, 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct SemaphoreCreateInfo<Next = Null> {
    pub stype: SemaphoreCreateInfoType,
    pub next: Next,
    pub flags: SemaphoreCreateFlags,
}
structure_type!(SemaphoreCreateInfoType, 9);

#[repr(C)]
#[derive(Debug, Default)]
pub struct CommandPoolCreateInfo<Next = Null> {
    pub stype: CommandPoolCreateInfoType,
    pub next: Next,
    pub flags: CommandPoolCreateFlags,
    pub queue_family_index: u32,
}
structure_type!(CommandPoolCreateInfoType, 39);

#[repr(C)]
#[derive(Debug)]
pub struct CommandBufferAllocateInfo<'a, Next = Null> {
    pub stype: CommandBufferAllocateInfoType,
    pub next: Next,
    pub pool: Mut<'a, VkCommandPool>,
    pub level: CommandBufferLevel,
    pub count: u32,
}
structure_type!(CommandBufferAllocateInfoType, 40);

#[repr(C)]
#[derive(Debug, Default)]
pub struct CommandBufferBeginInfo<Next = Null> {
    pub stype: CommandBufferBeginInfoType,
    pub next: Next,
    pub flags: CommandBufferUsageFlags,
    pub inheritance_info: Null, //TODO
}
structure_type!(CommandBufferBeginInfoType, 42);

#[repr(C)]
#[derive(Debug)]
pub struct MetalSurfaceCreateInfoEXT<Next = Null> {
    pub stype: MetalSurfaceCreateInfoEXTType,
    pub next: Next,
    pub flags: MetalSurfaceCreateFlagsEXT,
    pub layer: NonNull<c_void>,
}
structure_type!(MetalSurfaceCreateInfoEXTType, 1000217000);

#[repr(C)]
#[derive(Debug)]
pub struct SurfaceCapabilitiesKHR {
    pub min_image_count: u32,
    pub max_image_count: u32,
    pub current_extent: Extent2D,
    pub min_image_extent: Extent2D,
    pub max_image_extent: Extent2D,
    pub max_image_array_layers: u32,
    pub supported_transforms: SurfaceTransformFlagsKHR,
    pub current_transform: SurfaceTransformKHR,
    pub supported_composite_alpha: CompositeAlphaFlagsKHR,
    pub supported_usage_flags: ImageUsageFlags,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct SurfaceFormatKHR {
    pub format: Format,
    pub color_space: ColorSpaceKHR,
}

#[repr(C)]
#[derive(Debug)]
pub struct VkSwapchainCreateInfoKHR<'a, Next = Null> {
    pub stype: SwapchainCreateInfoKHRType,
    pub next: Next,
    pub flags: SwapchainCreateFlagsKHR,
    pub surface: Mut<'a, VkSurfaceKHR>,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub image_array_layers: u32,
    pub image_usage: ImageUsageFlags,
    pub image_sharing_mode: SharingMode,
    pub queue_family_indices: Slice<'a, u32>,
    pub pre_transform: SurfaceTransformKHR,
    pub composite_alpha: CompositeAlphaKHR,
    pub present_mode: PresentModeKHR,
    pub clipped: Bool,
    pub old_swapchain: Option<Mut<'a, VkSwapchainKHR>>,
}
structure_type!(SwapchainCreateInfoKHRType, 1000001000);

#[repr(C)]
#[derive(Debug)]
pub struct PresentInfoKHR<'a, Next = Null> {
    pub stype: PresentInfoType,
    pub next: Next,
    pub wait: Slice<'a, Mut<'a, VkSemaphore>>,
    /// Safety: The following members are arrays of the same length
    pub swapchains: Slice<'a, Mut<'a, VkSwapchainKHR>>,
    pub indices: Array<'a, u32>,
    pub results: Option<ArrayMut<'a, VkResult>>,
}
structure_type!(PresentInfoType, 1000001001);
