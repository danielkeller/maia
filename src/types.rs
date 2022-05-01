use crate::enums::*;
use crate::ffi::*;
pub use std::sync::Arc;

use std::num::NonZeroI32;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct Error(pub NonZeroI32);
const fn err(code: i32) -> Error {
    match NonZeroI32::new(code) {
        Some(i) => Error(i),
        None => panic!("Error code cannot be 0"),
    }
}
impl Error {
    pub const ERROR_OUT_OF_HOST_MEMORY: Error = err(-1);
    pub const INITIALIZATION_FAILED: Error = err(-3);
    pub const EXTENSION_NOT_PRESENT: Error = err(-7);
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f) // TODO
    }
}
impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

// These checks allow vulkan functions that return VkResult to return a
// Result<()> and use '?'. Note that this only works for that exact type.
const _: () = assert!(std::mem::size_of::<Result<()>>() == 4);
const _: () =
    assert!(unsafe { std::mem::transmute::<i32, Result<()>>(0).is_ok() });
const _: () = assert!(matches!(
    unsafe { std::mem::transmute::<i32, Result<()>>(-1) },
    Err(Error::ERROR_OUT_OF_HOST_MEMORY)
));

macro_rules! handle_debug {
    ($name: ident) => {
        impl std::fmt::Debug for $name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self._value).finish()
            }
        }
    };
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct InstanceRef<'a> {
    _value: NonNull<c_void>,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(InstanceRef);

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct PhysicalDeviceRef<'a> {
    _value: NonNull<c_void>,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(PhysicalDeviceRef);

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DeviceRef<'a> {
    _value: NonNull<c_void>,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(DeviceRef);

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct QueueRef<'a> {
    _value: NonNull<c_void>,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(QueueRef);

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SurfaceKHRRef<'a> {
    _value: NonNullNonDispatchableHandle,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(SurfaceKHRRef);

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SwapchainKHRRef<'a> {
    _value: NonNullNonDispatchableHandle,
    _lt: PhantomData<&'a ()>,
}
handle_debug!(SwapchainKHRRef);

pub(crate) type NonNullNonDispatchableHandle = std::num::NonZeroU64;

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
#[derive(Debug, Clone, Copy)]
pub struct Extent2D {
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Extent3D {
    width: u32,
    height: u32,
    depth: u32,
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
