use std::collections::HashSet;
use std::mem::MaybeUninit;
use std::ops::Range;

use crate::descriptor_set::DescriptorSetLayout;
use crate::device::Device;
use crate::enums::{PipelineLayoutCreateFlags, ShaderStageFlags};
use crate::error::{Error, Result};
use crate::ffi::*;
use crate::render_pass::RenderPass;
use crate::types::*;

/// A
#[doc = crate::spec_link!("pipeline layout", "descriptorsets-pipelinelayout")]
///
/// Create with [Device::create_pipeline_layout].
#[derive(Debug)]
pub struct PipelineLayout {
    handle: Handle<VkPipelineLayout>,
    set_layouts: Vec<Arc<DescriptorSetLayout>>,
    push_constant_ranges: Vec<PushConstantRange>,
    push_constant_voids: Vec<Range<u32>>,
    device: Arc<Device>,
}

impl Device {
    #[doc = crate::man_link!(vkCreatePipelineLayout)]
    pub fn create_pipeline_layout(
        self: &Arc<Self>,
        flags: PipelineLayoutCreateFlags,
        set_layouts: Vec<Arc<DescriptorSetLayout>>,
        push_constant_ranges: Vec<PushConstantRange>,
    ) -> Result<Arc<PipelineLayout>> {
        for range in &push_constant_ranges {
            // TODO: Check device limits
            if range.offset.overflowing_add(range.size).1 {
                return Err(Error::OutOfBounds);
            }
        }
        let mut handle = None;
        unsafe {
            let set_layouts =
                &set_layouts.iter().map(|l| l.handle()).collect::<Vec<_>>();
            (self.fun.create_pipeline_layout)(
                self.handle(),
                &PipelineLayoutCreateInfo {
                    flags,
                    set_layouts: set_layouts.into(),
                    push_constant_ranges: slice(&push_constant_ranges),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        let push_constant_voids = find_voids(&push_constant_ranges)?;
        Ok(Arc::new(PipelineLayout {
            handle: handle.unwrap(),
            set_layouts,
            push_constant_ranges,
            push_constant_voids,
            device: self.clone(),
        }))
    }
}

fn find_voids(ranges: &[PushConstantRange]) -> Result<Vec<Range<u32>>> {
    let mut result = vec![0..u32::MAX];
    for range in ranges {
        let end =
            range.offset.checked_add(range.size).ok_or(Error::OutOfBounds)?;
        let mut result1 = vec![];
        for void in result {
            if range.offset > void.start && end < void.end {
                result1.push(end..void.end);
                result1.push(void.start..range.offset);
            } else if range.offset > void.start {
                result1.push(void.start..range.offset.min(void.end));
            } else {
                result1.push(void.start.max(end)..void.end);
            }
        }
        result = result1;
    }
    Ok(result)
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline_layout)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PipelineLayout {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkPipelineLayout> {
        self.handle.borrow()
    }
    /// Returns the list of descriptor set layouts.
    pub fn layouts(&self) -> &[Arc<DescriptorSetLayout>] {
        &self.set_layouts
    }
    /// Checks that the push constants are in bounds and 'stage_flags' are
    /// correct.
    pub fn bounds_check_push_constants(
        &self,
        stage_flags: ShaderStageFlags,
        offset: u32,
        size: u32,
    ) -> bool {
        let (end, overflow) = offset.overflowing_add(size);
        if overflow {
            return false;
        }
        for void in &self.push_constant_voids {
            if void.start < end && offset < void.end {
                return false;
            }
        }
        for range in &self.push_constant_ranges {
            if range.offset < end
                && offset < range.offset + range.size
                && stage_flags & range.stage_flags != range.stage_flags
            {
                return false;
            }
        }
        return true;
    }
}

/// A
#[doc = crate::spec_link!("pipeline", "pipelines")]
#[derive(Debug)]
///
/// Create with [Device::create_graphics_pipeline()] or
/// [Device::create_compute_pipeline()].
pub struct Pipeline {
    handle: Handle<VkPipeline>,
    layout: Arc<PipelineLayout>,
    render_pass: Option<Arc<RenderPass>>,
    subpass: u32,
    device: Arc<Device>,
}

#[doc = crate::man_link!(VkGraphicsPipelineCreateInfo)]
pub struct GraphicsPipelineCreateInfo<'a> {
    pub stages: &'a [PipelineShaderStageCreateInfo<'a>],
    pub vertex_input_state: &'a PipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_state: &'a PipelineInputAssemblyStateCreateInfo,
    pub tessellation_state: Option<&'a PipelineTessellationStateCreateInfo>,
    pub viewport_state: &'a PipelineViewportStateCreateInfo<'a>,
    pub rasterization_state: &'a PipelineRasterizationStateCreateInfo,
    pub multisample_state: &'a PipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_state: Option<&'a PipelineDepthStencilStateCreateInfo>,
    pub color_blend_state: &'a PipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_state: Option<&'a PipelineDynamicStateCreateInfo<'a>>,
    pub layout: &'a Arc<PipelineLayout>,
    pub render_pass: &'a Arc<RenderPass>,
    pub subpass: u32,
    pub cache: Option<&'a PipelineCache>,
}

impl Device {
    // TODO: Bulk create
    /// Returns [Error::OutOfBounds] if 'info.subpass' is out of bounds of
    /// 'info.render_pass', or the specialization constants are out of bounds.
    /// Returns [Error::InvalidArgument] if any vertex input binding number are
    /// repeated, any vertex attribute locations are repeated, or any vertex
    /// attributes refer to a nonexistent binding.
    #[doc = crate::man_link!(vkCreateGraphicsPipeline)]
    pub fn create_graphics_pipeline(
        self: &Arc<Self>,
        info: &GraphicsPipelineCreateInfo,
    ) -> Result<Arc<Pipeline>> {
        if info.subpass >= info.render_pass.num_subpasses() {
            return Err(Error::OutOfBounds);
        }
        let mut bindings = HashSet::new();
        for b in info.vertex_input_state.vertex_binding_descriptions {
            if !bindings.insert(b.binding) {
                return Err(Error::InvalidArgument);
            }
        }
        let mut locations = HashSet::new();
        for att in info.vertex_input_state.vertex_attribute_descriptions {
            if !locations.insert(att.location)
                || !bindings.contains(&att.binding)
            {
                return Err(Error::InvalidArgument);
            }
        }
        for stage in info.stages {
            check_specialization_constants(&stage)?;
        }
        let vk_info = VkGraphicsPipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stages: info.stages.into(),
            vertex_input_state: info.vertex_input_state,
            input_assembly_state: info.input_assembly_state,
            tessellation_state: info.tessellation_state,
            viewport_state: info.viewport_state,
            rasterization_state: info.rasterization_state,
            multisample_state: info.multisample_state,
            depth_stencil_state: info.depth_stencil_state,
            color_blend_state: info.color_blend_state,
            dynamic_state: info.dynamic_state,
            layout: info.layout.handle(),
            render_pass: info.render_pass.handle(),
            subpass: info.subpass,
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (self.fun.create_graphics_pipelines)(
                self.handle(),
                info.cache.map(|c| c.handle.borrow()),
                1,
                std::array::from_ref(&vk_info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            layout: info.layout.clone(),
            render_pass: Some(info.render_pass.clone()),
            subpass: info.subpass,
            device: self.clone(),
        }))
    }
    /// Returns [Error::OutOfBounds] if the specialization constants are out of
    /// bounds.
    #[doc = crate::man_link!(vkCreateComputePipeline)]
    pub fn create_compute_pipeline(
        self: &Arc<Self>,
        stage: PipelineShaderStageCreateInfo,
        layout: &Arc<PipelineLayout>,
        cache: Option<&PipelineCache>,
    ) -> Result<Arc<Pipeline>> {
        check_specialization_constants(&stage)?;
        let info = ComputePipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage,
            layout: layout.handle(),
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (self.fun.create_compute_pipelines)(
                self.handle(),
                cache.map(|c| c.handle.borrow()),
                1,
                std::array::from_ref(&info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            layout: layout.clone(),
            render_pass: None,
            subpass: 0,
            device: self.clone(),
        }))
    }
}

impl Pipeline {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkPipeline> {
        self.handle.borrow()
    }
    /// Returns the pipeline layout.
    pub fn layout(&self) -> &PipelineLayout {
        &*self.layout
    }
    /// Returns the render pass the pipeline was created with, if it is a
    /// graphics pipeline.
    pub fn render_pass(&self) -> Option<&RenderPass> {
        self.render_pass.as_deref()
    }
    /// Returns true if the pipeline is compatible with the given render pass
    /// and subpass.
    pub fn is_compatible_with(&self, pass: &RenderPass, subpass: u32) -> bool {
        self.render_pass.as_ref().map_or(false, |p| p.compatible(pass))
            && self.subpass == subpass
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

fn check_specialization_constants<T>(
    info: &PipelineShaderStageCreateInfo<T>,
) -> Result<()> {
    if let Some(spec) = &info.specialization_info {
        for entry in spec.map_entries {
            if spec.data.len() < entry.offset as usize
                || spec.data.len() - (entry.offset as usize) < entry.size
            {
                return Err(Error::OutOfBounds);
            }
        }
    }
    Ok(())
}

/// A
#[doc = crate::spec_link!("pipeline cache", "pipelines-cache")]
///
/// Create with [Device::create_pipeline_cache()]
pub struct PipelineCache {
    handle: Handle<VkPipelineCache>,
    device: Arc<Device>,
}

impl Device {
    /// Safety: 'data' must either be empty or have been retuned from a previous
    /// call to [PipelineCache::data()]. Hilariously, this function is
    /// actually impossible to make safe; Vulkan provides no way to validate the
    /// cache data, and the data is generally written to a file where it could
    /// be damaged or altered. Caveat emptor.
    ///
    #[doc = crate::man_link!(vkCreatePipelineCache)]
    pub unsafe fn create_pipeline_cache(
        self: &Arc<Self>,
        data: &[u8],
    ) -> Result<PipelineCache> {
        let mut handle = None;
        let info = PipelineCacheCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            initial_data: data.into(),
        };
        (self.fun.create_pipeline_cache)(
            self.handle(),
            &info,
            None,
            &mut handle,
        )?;
        Ok(PipelineCache { handle: handle.unwrap(), device: self.clone() })
    }
}

impl Drop for PipelineCache {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline_cache)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PipelineCache {
    /// Returns the data in the pipeline cache.
    pub fn data(&self) -> Result<Vec<u8>> {
        let mut len = 0;
        let mut result = Vec::new();
        loop {
            unsafe {
                (self.device.fun.get_pipeline_cache_data)(
                    self.device.handle(),
                    self.handle.borrow(),
                    &mut len,
                    None,
                )?;
                result.reserve(len);
                let maybe_worked = (self.device.fun.get_pipeline_cache_data)(
                    self.device.handle(),
                    self.handle.borrow(),
                    &mut len,
                    ArrayMut::from_slice(result.spare_capacity_mut()),
                );
                if let Err(err) = maybe_worked {
                    if let Error::Incomplete = err.into() {
                        continue; // Racing pipeline creation
                    }
                }
                maybe_worked?;
                break;
            }
        }
        unsafe { result.set_len(len) };
        Ok(result)
    }
}
