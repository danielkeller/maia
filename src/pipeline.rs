use std::collections::HashSet;
use std::mem::MaybeUninit;

use crate::descriptor_set::DescriptorSetLayout;
use crate::device::Device;
use crate::enums::PipelineLayoutCreateFlags;
use crate::error::{Error, Result};
use crate::ffi::*;
use crate::render_pass::RenderPass;
use crate::types::*;

pub struct PipelineLayout {
    handle: Handle<VkPipelineLayout>,
    set_layouts: Vec<Arc<DescriptorSetLayout>>,
    device: Arc<Device>,
}

impl Device {
    pub fn create_pipeline_layout(
        self: &Arc<Self>,
        flags: PipelineLayoutCreateFlags,
        set_layouts: Vec<Arc<DescriptorSetLayout>>,
        push_constant_ranges: &[PushConstantRange],
    ) -> Result<Arc<PipelineLayout>> {
        let mut handle = None;
        unsafe {
            let set_layouts =
                &set_layouts.iter().map(|l| l.borrow()).collect::<Vec<_>>();
            (self.fun.create_pipeline_layout)(
                self.borrow(),
                &PipelineLayoutCreateInfo {
                    flags,
                    set_layouts: set_layouts.into(),
                    push_constant_ranges: Slice::from(push_constant_ranges),
                    ..Default::default()
                },
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(PipelineLayout {
            handle: handle.unwrap(),
            set_layouts,
            device: self.clone(),
        }))
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline_layout)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl PipelineLayout {
    pub fn borrow(&self) -> Ref<VkPipelineLayout> {
        self.handle.borrow()
    }
    pub fn layout(&self, binding: u32) -> Option<&Arc<DescriptorSetLayout>> {
        self.set_layouts.get(binding as usize)
    }
}

#[derive(Debug)]
pub struct Pipeline {
    handle: Handle<VkPipeline>,
    device: Arc<Device>,
}

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
    pub layout: &'a PipelineLayout,
    pub render_pass: &'a RenderPass,
    pub subpass: u32,
}

impl Device {
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
        let info = VkGraphicsPipelineCreateInfo {
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
            layout: info.layout.borrow(),
            render_pass: info.render_pass.borrow(),
            subpass: info.subpass,
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (self.fun.create_graphics_pipelines)(
                self.borrow(),
                None,
                1,
                std::array::from_ref(&info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            device: self.clone(),
        }))
    }
    pub fn create_compute_pipeline(
        self: &Arc<Self>,
        stage: PipelineShaderStageCreateInfo,
        layout: &PipelineLayout,
    ) -> Result<Arc<Pipeline>> {
        let info = ComputePipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stage,
            layout: layout.borrow(),
            base_pipeline_handle: Default::default(),
            base_pipeline_index: Default::default(),
        };
        let mut handle = MaybeUninit::uninit();
        unsafe {
            (self.fun.create_compute_pipelines)(
                self.borrow(),
                None,
                1,
                std::array::from_ref(&info).into(),
                None,
                std::array::from_mut(&mut handle).into(),
            )?;
        }
        Ok(Arc::new(Pipeline {
            handle: unsafe { handle.assume_init() },
            device: self.clone(),
        }))
    }
}

impl Pipeline {
    pub fn borrow(&self) -> Ref<VkPipeline> {
        self.handle.borrow()
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_pipeline)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}
