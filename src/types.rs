use crate::enums::*;
use crate::error::Error;
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
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct NonNullDispatchableHandle(NonNull<c_void>);
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(PartialEq, Eq, Hash)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkInstance(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkPhysicalDevice(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkDevice(NonNullDispatchableHandle);
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkQueue(NonNullDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkSemaphore(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkFence(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkSampler(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkDescriptorSetLayout(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkPipelineLayout(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkPipelineCache(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkPipeline(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkBuffer(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkImage(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkImageView(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkFramebuffer(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkRenderPass(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkShaderModule(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkCommandPool(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkCommandBuffer(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkSurfaceKHR(NonNullNonDispatchableHandle);

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Copy, Default)]
pub struct Offset2D {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect2D {
    pub offset: Offset2D,
    pub extent: Extent2D,
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
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
    pub robust_buffer_access: Bool,
    pub full_draw_index_uint32: Bool,
    pub image_cube_array: Bool,
    pub independent_blend: Bool,
    pub geometry_shader: Bool,
    pub tessellation_shader: Bool,
    pub sample_rate_shading: Bool,
    pub dual_src_blend: Bool,
    pub logic_op: Bool,
    pub multi_draw_indirect: Bool,
    pub draw_indirect_first_instance: Bool,
    pub depth_clamp: Bool,
    pub depth_bias_clamp: Bool,
    pub fill_mode_non_solid: Bool,
    pub depth_bounds: Bool,
    pub wide_lines: Bool,
    pub large_points: Bool,
    pub alpha_to_one: Bool,
    pub multi_viewport: Bool,
    pub sampler_anisotropy: Bool,
    pub texture_compression_etc2: Bool,
    pub texture_compression_astc_ldr: Bool,
    pub texture_compression_bc: Bool,
    pub occlusion_query_precise: Bool,
    pub pipeline_statistics_query: Bool,
    pub vertex_pipeline_stores_and_atomics: Bool,
    pub fragment_stores_and_atomics: Bool,
    pub shader_tessellation_and_geometry_point_size: Bool,
    pub shader_image_gather_extended: Bool,
    pub shader_storage_image_extended_formats: Bool,
    pub shader_storage_image_multisample: Bool,
    pub shader_storage_image_read_without_format: Bool,
    pub shader_storage_image_write_without_format: Bool,
    pub shader_uniform_buffer_array_dynamic_indexing: Bool,
    pub shader_sampled_image_array_dynamic_indexing: Bool,
    pub shader_storage_buffer_array_dynamic_indexing: Bool,
    pub shader_storage_image_array_dynamic_indexing: Bool,
    pub shader_clip_distance: Bool,
    pub shader_cull_distance: Bool,
    pub shader_float64: Bool,
    pub shader_int64: Bool,
    pub shader_int16: Bool,
    pub shader_resource_residency: Bool,
    pub shader_resource_min_lod: Bool,
    pub sparse_binding: Bool,
    pub sparse_residency_buffer: Bool,
    pub sparse_residency_image_2d: Bool,
    pub sparse_residency_image_3d: Bool,
    pub sparse_residency2_samples: Bool,
    pub sparse_residency4_samples: Bool,
    pub sparse_residency8_samples: Bool,
    pub sparse_residency16_samples: Bool,
    pub sparse_residency_aliased: Bool,
    pub variable_multisample_rate: Bool,
    pub inherited_queries: Bool,
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
#[derive(Debug)]
pub struct VkImageViewCreateInfo<'a, Next = Null> {
    pub stype: ImageViewCreateInfoType,
    pub next: Next,
    pub flags: ImageViewCreateFlags,
    pub image: Ref<'a, VkImage>,
    pub view_type: ImageViewType,
    pub format: Format,
    pub components: ComponentMapping,
    pub subresource_range: ImageSubresourceRange,
}
structure_type!(ImageViewCreateInfoType, 15);

#[repr(C)]
#[derive(Debug)]
pub struct VkShaderModuleCreateInfo<'a, Next = Null> {
    pub stype: ShaderModuleCreateInfoType,
    pub next: Next,
    pub flags: ShaderModuleCreateFlags,
    pub code: Bytes<'a>,
}
structure_type!(ShaderModuleCreateInfoType, 16);

#[repr(C)]
#[derive(Debug)]
pub struct SpecializationMapEntry {
    pub constant_id: u32,
    pub offset: u32,
    pub size: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct SpecializationInfo<'a> {
    pub map_entries: Slice<'a, SpecializationMapEntry>,
    pub data: Bytes<'a>,
}

#[repr(C)]
#[derive(Debug)]
pub struct PipelineShaderStageCreateInfo<'a, Next = Null> {
    pub stype: PipelineShaderStageCreateInfoType,
    pub next: Next,
    pub flags: PipelineShaderStageCreateFlags,
    pub stage: ShaderStage,
    pub module: Ref<'a, VkShaderModule>,
    pub name: Str<'a>,
    pub specialization_info: Option<&'a SpecializationInfo<'a>>,
}
structure_type!(PipelineShaderStageCreateInfoType, 18);

impl<'a> PipelineShaderStageCreateInfo<'a> {
    const MAIN: Str<'static> = unsafe { Str::new_unchecked(b"main\0") };
    /// Create a vertex shader with entry point "main"
    pub fn vertex(module: &'a crate::shader::ShaderModule) -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage: ShaderStage::VERTEX,
            module: module.borrow(),
            name: Self::MAIN,
            specialization_info: None,
        }
    }
    /// Create a fragment shader with entry point "main"
    pub fn fragment(module: &'a crate::shader::ShaderModule) -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage: ShaderStage::FRAGMENT,
            module: module.borrow(),
            name: Self::MAIN,
            specialization_info: None,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct VertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

#[repr(C)]
#[derive(Debug)]
pub struct VertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: Format,
    pub offset: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineVertexInputStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineVertexInputStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineVertexInputStateCreateFlags,
    pub vertex_binding_descriptions: Slice_<'a, VertexInputBindingDescription>,
    pub vertex_attribute_descriptions:
        Slice<'a, VertexInputAttributeDescription>,
}
structure_type!(PipelineVertexInputStateCreateInfoType, 19);

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineInputAssemblyStateCreateInfo<Next = Null> {
    pub stype: PipelineInputAssemblyStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineInputAssemblyStateCreateFlags,
    pub topology: PrimitiveTopology,
    pub primitive_restart_enable: Bool,
}
structure_type!(PipelineInputAssemblyStateCreateInfoType, 20);

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineTessellationStateCreateInfo<Next = Null> {
    pub stype: PipelineTesselationStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineTesselationStateCreateFlags,
    pub patch_control_points: u32,
}
structure_type!(PipelineTesselationStateCreateInfoType, 21);

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineViewportStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineViewportStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineViewportStateCreateFlags,
    pub viewports: Slice_<'a, Viewport>,
    pub scissors: Slice<'a, Rect2D>,
}
structure_type!(PipelineViewportStateCreateInfoType, 22);

#[repr(C)]
#[derive(Debug)]
pub struct PipelineRasterizationStateCreateInfo<Next = Null> {
    pub stype: PipelineRasterizationStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: Bool,
    pub rasterizer_discard_enable: Bool,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullModeFlags,
    pub front_face: FrontFace,
    pub depth_bias_enable: Bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}
structure_type!(PipelineRasterizationStateCreateInfoType, 23);

impl Default for PipelineRasterizationStateCreateInfo {
    fn default() -> Self {
        PipelineRasterizationStateCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            depth_clamp_enable: Bool::False,
            rasterizer_discard_enable: Bool::False,
            polygon_mode: PolygonMode::FILL,
            cull_mode: CullModeFlags::NONE,
            front_face: FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: Bool::False,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineMultisampleStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineMultisampleStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineMultisampleStateCreateFlags,
    pub rasterization_samples: SampleCount,
    pub sample_shading_enable: Bool,
    pub min_sample_shading: f32,
    pub sample_mask: Option<&'a u64>,
    pub alpha_to_coverage_enable: Bool,
    pub alpha_to_one_enable: Bool,
}
structure_type!(PipelineMultisampleStateCreateInfoType, 24);

#[repr(C)]
#[derive(Debug, Default)]
pub struct StencilOpState {
    pub fail_op: StencilOp,
    pub pass_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub compare_op: CompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineDepthStencilStateCreateInfo<Next = Null> {
    pub stype: PipelineDepthStencilStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: Bool,
    pub depth_write_enable: Bool,
    pub depth_compare_op: CompareOp,
    pub depth_bounds_test_enable: Bool,
    pub stencil_test_enable: Bool,
    pub front: StencilOpState,
    pub back: StencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}
structure_type!(PipelineDepthStencilStateCreateInfoType, 25);

#[repr(C)]
#[derive(Debug)]
pub struct PipelineColorBlendAttachmentState {
    pub blend_enable: Bool,
    pub src_color_blend_factor: BlendFactor,
    pub dst_color_blend_factor: BlendFactor,
    pub color_blend_op: BlendOp,
    pub src_alpha_blend_factor: BlendFactor,
    pub dst_alpha_blend_factor: BlendFactor,
    pub alpha_blend_op: BlendOp,
    pub color_write_mask: ColorComponentFlags,
}

const DEFAULT_BLEND: PipelineColorBlendAttachmentState =
    PipelineColorBlendAttachmentState {
        blend_enable: Bool::False,
        src_color_blend_factor: BlendFactor::ONE,
        dst_color_blend_factor: BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: BlendOp::ADD,
        src_alpha_blend_factor: BlendFactor::ONE,
        dst_alpha_blend_factor: BlendFactor::ONE_MINUS_SRC_ALPHA,
        alpha_blend_op: BlendOp::ADD,
        color_write_mask: ColorComponentFlags::RGBA,
    };

impl Default for PipelineColorBlendAttachmentState {
    /// Blending disabled, and premultiplied alpha blending parameters.
    fn default() -> Self {
        DEFAULT_BLEND
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct PipelineColorBlendStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineColorBlendStateCreateInfoType,
    pub next: Next,
    pub flags: PipelineColorBlendStateCreateFlags,
    pub logic_op_enable: Bool,
    pub logic_op: LogicOp,
    pub attachments: Slice_<'a, PipelineColorBlendAttachmentState>,
    pub blend_constants: [f32; 4],
}
structure_type!(PipelineColorBlendStateCreateInfoType, 26);

impl Default for PipelineColorBlendStateCreateInfo<'static> {
    /// One color attachment with blending disabled
    fn default() -> Self {
        Self {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            logic_op_enable: Bool::False,
            logic_op: LogicOp::CLEAR,
            attachments: std::slice::from_ref(&DEFAULT_BLEND).into(),
            blend_constants: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineDynamicStateCreateInfo<'a, Next = Null> {
    pub stype: PipelineDynamicStateCreateInfoType,
    pub next: Next,
    pub flags: DynamicStateCreateFlags,
    pub dynamic_states: Slice_<'a, DynamicState>,
}
structure_type!(PipelineDynamicStateCreateInfoType, 27);

#[repr(C)]
#[derive(Debug)]
pub struct GraphicsPipelineCreateInfo<'a, Next = Null> {
    pub stype: GraphicsPipelineCreateInfoType,
    pub next: Next,
    pub flags: PipelineCreateFlags,
    pub stages: Slice_<'a, PipelineShaderStageCreateInfo<'a>>,
    pub vertex_input_state: &'a PipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a PipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: Option<&'a PipelineTessellationStateCreateInfo>,
    pub viewport_state: &'a PipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a PipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: Option<&'a PipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: &'a PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: Option<&'a PipelineDynamicStateCreateInfo<'a>>,
    pub layout: Ref<'a, VkPipelineLayout>,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub subpass: u32,
    pub base_pipeline_handle: Option<Ref<'a, VkPipeline>>,
    pub base_pipeline_index: u32,
}
structure_type!(GraphicsPipelineCreateInfoType, 28);

#[repr(C)]
#[derive(Debug)]
pub struct PushConstantRange {
    pub stage_flags: ShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PipelineLayoutCreateInfo<'a, Next = Null> {
    pub stype: PipelineLayoutCreateInfoType,
    pub next: Next,
    pub flags: PipelineLayoutCreateFlags,
    pub set_layouts: Slice_<'a, Ref<'a, VkDescriptorSetLayout>>,
    pub push_constant_ranges: Slice<'a, PushConstantRange>,
}
structure_type!(PipelineLayoutCreateInfoType, 30);

#[repr(C)]
#[derive(Debug)]
pub struct VkDescriptorSetLayoutBinding<'a> {
    pub binding: u32,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: ShaderStageFlags,
    // Safety: Must be descriptor_count long
    pub immutable_samplers: Option<Array<'a, VkSampler>>,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct VkDescriptorSetLayoutCreateInfo<'a, Next = Null> {
    pub stype: DescriptorSetLayoutCreateInfoType,
    pub next: Next,
    pub flags: DescriptorSetLayoutCreateFlags,
    pub bindings: Slice_<'a, VkDescriptorSetLayoutBinding<'a>>,
}
structure_type!(DescriptorSetLayoutCreateInfoType, 32);

#[repr(C)]
#[derive(Debug, Default)]
pub struct AttachmentDescription {
    pub flags: AttachmentDescriptionFlags,
    pub format: Format,
    pub samples: SampleCount,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub stencil_load_op: AttachmentLoadOp,
    pub stencil_store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct AttachmentReference {
    pub attachment: u32,
    pub layout: ImageLayout,
}

#[repr(C)]
#[derive(Debug)]
pub struct VkSubpassDescription<'a> {
    flags: SubpassDescriptionFlags,
    pipeline_bind_point: PipelineBindPoint,
    pub(crate) input_attachments: Slice<'a, AttachmentReference>,
    pub(crate) color_attachments: Slice<'a, AttachmentReference>,
    /// Safety: Must be the same length as color_attachments
    pub(crate) resolve_attachments: Option<Array<'a, AttachmentReference>>,
    pub(crate) depth_stencil_attachments:
        Option<Array<'a, AttachmentReference>>,
    pub(crate) preserve_attachments: Slice<'a, AttachmentReference>,
}

#[derive(Default)]
pub struct SubpassDescription<'a> {
    pub flags: SubpassDescriptionFlags,
    pub pipeline_bind_point: PipelineBindPoint,
    pub input_attachments: &'a [AttachmentReference],
    pub color_attachments: &'a [AttachmentReference],
    /// Must be either empty or the same length as color_attachments
    pub resolve_attachments: &'a [AttachmentReference],
    /// Must be either empty or the same length as color_attachments
    pub depth_stencil_attachments: &'a [AttachmentReference],
    pub preserve_attachments: &'a [AttachmentReference],
}
impl<'a> TryFrom<SubpassDescription<'a>> for VkSubpassDescription<'a> {
    type Error = Error;
    #[inline]
    fn try_from(value: SubpassDescription<'a>) -> Result<Self, Self::Error> {
        if !value.resolve_attachments.is_empty()
            && value.resolve_attachments.len() != value.color_attachments.len()
        {
            return Err(Error::InvalidArgument);
        }
        if !value.depth_stencil_attachments.is_empty()
            && value.depth_stencil_attachments.len()
                != value.color_attachments.len()
        {
            return Err(Error::InvalidArgument);
        }
        Ok(Self {
            flags: value.flags,
            pipeline_bind_point: value.pipeline_bind_point,
            input_attachments: value.input_attachments.into(),
            color_attachments: value.color_attachments.into(),
            resolve_attachments: Array::from_slice(value.resolve_attachments),
            depth_stencil_attachments: Array::from_slice(
                value.depth_stencil_attachments,
            ),
            preserve_attachments: value.preserve_attachments.into(),
        })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: PipelineStageFlags,
    pub dst_stage_mask: PipelineStageFlags,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub dependency_flags: DependencyFlags,
}

#[repr(C)]
#[derive(Debug)]
pub struct VkFramebufferCreateInfo<'a, Next = Null> {
    pub stype: FramebufferCreateInfoType,
    pub next: Next,
    pub flags: FramebufferCreateFlags,
    pub render_pass: Ref<'a, VkRenderPass>,
    pub attachments: Slice<'a, Ref<'a, VkImageView>>,
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}
structure_type!(FramebufferCreateInfoType, 37);

#[repr(C)]
#[derive(Debug, Default)]
pub struct RenderPassCreateInfo<'a, Next = Null> {
    pub stype: RenderPassCreateInfoType,
    pub next: Next,
    pub flags: RenderPassCreateFlags,
    pub attachments: Slice_<'a, AttachmentDescription>,
    pub subpasses: Slice<'a, VkSubpassDescription<'a>>,
    pub dependencies: Slice<'a, SubpassDependency>,
}
structure_type!(RenderPassCreateInfoType, 38);

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
pub struct VkBufferMemoryBarrier<'a, Next = Null> {
    pub stype: BufferMemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub buffer: Ref<'a, VkBuffer>,
    pub offset: u64,
    pub size: u64,
}
structure_type!(BufferMemoryBarrierType, 44);

#[repr(C)]
#[derive(Debug)]
pub struct VkImageMemoryBarrier<'a, Next = Null> {
    pub stype: ImageMemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
    pub old_layout: ImageLayout,
    pub new_layout: ImageLayout,
    pub src_queue_family_index: u32,
    pub dst_queue_family_index: u32,
    pub image: Ref<'a, VkImage>,
    pub subresource_range: ImageSubresourceRange,
}
structure_type!(ImageMemoryBarrierType, 45);

#[repr(C)]
#[derive(Debug, Default)]
pub struct MemoryBarrier<Next = Null> {
    pub stype: MemoryBarrierType,
    pub next: Next,
    pub src_access_mask: AccessFlags,
    pub dst_access_mask: AccessFlags,
}
structure_type!(MemoryBarrierType, 46);

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
