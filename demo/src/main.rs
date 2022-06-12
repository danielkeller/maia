use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;
use std::time::Instant;

use ember::vk;
use inline_spirv::include_spirv;
use ultraviolet::{Mat4, Vec3};

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

const VERTEX_DATA: [Vertex; 4] = [
    Vertex {
        pos: [-0.7, -0.7, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        pos: [-0.7, 0.7, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 0.0],
    },
    Vertex {
        pos: [0.7, -0.7, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [0.7, 0.7, 0.0, 1.0],
        color: [0.3, 0.3, 0.3, 0.0],
    },
];
const INDEX_DATA: [u16; 6] = [0, 1, 2, 2, 1, 3];

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct UBO {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

fn required_instance_extensions() -> anyhow::Result<&'static [vk::Str<'static>]>
{
    let exts = vk::instance_extension_properties()?;
    if exts
        .iter()
        .any(|e| e.extension_name == vk::ext::GET_PHYSICAL_DEVICE_PROPERTIES2)
    {
        Ok(std::slice::from_ref(&vk::ext::GET_PHYSICAL_DEVICE_PROPERTIES2))
    } else {
        Ok(&[])
    }
}

fn pick_physical_device(
    mut phys: Vec<vk::PhysicalDevice>,
) -> vk::PhysicalDevice {
    for i in 0..phys.len() {
        if phys[i].properties().device_type
            == vk::PhysicalDeviceType::DISCRETE_GPU
        {
            return phys.swap_remove(i);
        }
    }
    for i in 0..phys.len() {
        if phys[i].properties().device_type
            == vk::PhysicalDeviceType::INTEGRATED_GPU
        {
            return phys.swap_remove(i);
        }
    }
    phys.swap_remove(0)
}

fn pick_queue_family(
    phy: &vk::PhysicalDevice,
    surf: &vk::ext::SurfaceKHR,
) -> anyhow::Result<u32> {
    let props = phy.queue_family_properties();
    for i in 0..props.len() {
        if !(props[i].queue_flags | vk::QueueFlags::GRAPHICS).is_empty()
            && surf.support(phy, i as u32)?
        {
            return Ok(i as u32);
        }
    }
    anyhow::bail!("No graphics queue")
}

fn required_device_extensions(
    phy: &vk::PhysicalDevice,
) -> anyhow::Result<&'static [vk::Str<'static>]> {
    let exts = phy.device_extension_properties()?;
    if exts.iter().any(|e| e.extension_name == vk::ext::PORTABILITY_SUBSET) {
        Ok(&[vk::ext::PORTABILITY_SUBSET, vk::ext::SWAPCHAIN])
    } else {
        Ok(&[vk::ext::SWAPCHAIN])
    }
}

fn memory_type(
    phy: &vk::PhysicalDevice,
    desired: vk::MemoryPropertyFlags,
) -> u32 {
    let mem_props = phy.memory_properties();
    for (num, props) in mem_props.memory_types.iter().enumerate() {
        if props.property_flags & desired == desired {
            return num as u32;
        }
    }
    panic!("No host visible memory!")
}

fn main() -> anyhow::Result<()> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;

    let mut instance_exts = vec![];
    instance_exts
        .extend(ember::window::required_instance_extensions(&window)?.iter());
    instance_exts.extend(required_instance_extensions()?.iter());
    let inst = vk::create_instance(&vk::InstanceCreateInfo {
        enabled_extension_names: instance_exts.as_slice().into(),
        ..Default::default()
    })?;

    let surf = ember::window::create_surface(&inst, &window)?;

    let phy = pick_physical_device(inst.enumerate_physical_devices()?);
    let queue_family = pick_queue_family(&phy, &surf)?;
    if !surf.surface_formats(&phy)?.iter().any(|f| {
        f == &vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR,
        }
    }) {
        anyhow::bail!("Desired surface format not found");
    }

    let device_extensions = required_device_extensions(&phy)?;
    let device = phy.create_device(&vk::DeviceCreateInfo {
        queue_create_infos: vk::Slice_::from_slice(&[
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family,
                queue_priorities: (&[1.0]).into(),
                ..Default::default()
            },
        ]),
        enabled_extension_names: device_extensions.into(),
        ..Default::default()
    })?;
    let mut queue = device.queue(0, 0)?;

    let mut acquire_sem = device.create_semaphore()?;
    let mut present_sem = device.create_semaphore()?;
    let mut fence = Some(device.create_fence()?);

    let mut swapchain_size = window.inner_size();
    let mut swapchain = Some(device.khr_swapchain().create(
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: vk::Extent2D {
                width: swapchain_size.width,
                height: swapchain_size.height,
            },
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?);

    let mut cmd_pool = device.create_command_pool(queue_family)?;

    let data_size = std::mem::size_of_val(&VERTEX_DATA)
        + std::mem::size_of_val(&INDEX_DATA);
    let index_offset = std::mem::size_of_val(&VERTEX_DATA);

    let vertex_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: data_size as u64,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::INDEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
        ..Default::default()
    })?;
    let mem_size = vertex_buffer.memory_requirements().size;
    let memory = device.allocate_memory(
        mem_size as u64,
        memory_type(&phy, vk::MemoryPropertyFlags::DEVICE_LOCAL),
    )?;
    let vertex_buffer = memory.bind_buffer_memory(vertex_buffer, 0)?;

    let staging_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: data_size as u64,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        ..Default::default()
    })?;
    let mem_size = staging_buffer.memory_requirements().size;
    let memory = device.allocate_memory(
        mem_size as u64,
        memory_type(
            &phy,
            vk::MemoryPropertyFlags::HOST_VISIBLE
                | vk::MemoryPropertyFlags::HOST_COHERENT,
        ),
    )?;
    let staging_buffer = memory.bind_buffer_memory(staging_buffer, 0)?;

    let mut memory = memory.map(0, data_size)?;
    *bytemuck::from_bytes_mut(&mut memory.slice_mut()[0..index_offset]) =
        VERTEX_DATA;
    *bytemuck::from_bytes_mut(&mut memory.slice_mut()[index_offset..]) =
        INDEX_DATA;
    memory.unmap();

    let transfer = cmd_pool.allocate()?;
    let mut rec = cmd_pool.begin(transfer)?;
    rec.copy_buffer(
        &staging_buffer,
        &vertex_buffer,
        &[vk::BufferCopy { size: mem_size, ..Default::default() }],
    )?;
    rec.memory_barrier(
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::TRANSFER_WRITE,
        vk::AccessFlags::VERTEX_ATTRIBUTE_READ | vk::AccessFlags::INDEX_READ,
    );
    let mut transfer = rec.end()?;
    let pending_fence = queue.submit(
        &mut [vk::SubmitInfo {
            commands: &mut [&mut transfer],
            ..Default::default()
        }],
        fence.take().unwrap(),
    )?;
    fence = Some(pending_fence.wait()?);

    let uniform_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: size_of::<UBO>() as u64,
        usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
        ..Default::default()
    })?;
    let mem_size = uniform_buffer.memory_requirements().size;
    let memory = device.allocate_memory(
        mem_size as u64,
        memory_type(
            &phy,
            vk::MemoryPropertyFlags::DEVICE_LOCAL
                | vk::MemoryPropertyFlags::HOST_VISIBLE,
        ),
    )?;
    let uniform_buffer = memory.bind_buffer_memory(uniform_buffer, 0)?;

    let mut uniform_memory = memory.map(0, size_of::<UBO>())?;

    let render_pass = device.create_render_pass(&vk::RenderPassCreateInfo {
        attachments: vk::Slice_::from_slice(&[vk::AttachmentDescription {
            format: vk::Format::B8G8R8A8_SRGB,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }]),
        subpasses: vk::Slice::from_slice(&[vk::SubpassDescription {
            color_attachments: &[vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            }],
            ..Default::default()
        }
        .try_into()?]),
        ..Default::default()
    })?;

    let vertex_shader = device
        .create_shader_module(include_spirv!("shaders/triangle.vert", vert))?;
    let fragment_shader = device
        .create_shader_module(include_spirv!("shaders/triangle.frag", frag))?;

    let descriptor_set_layout = device.create_descriptor_set_layout(vec![
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            immutable_samplers: vec![],
        },
    ])?;
    let mut descriptor_pool = device.create_descriptor_pool(
        1,
        &[vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
        }],
    )?;
    let mut desc_set = descriptor_pool.allocate(&descriptor_set_layout)?;

    let mut update = device.create_descriptor_set_update_builder();
    update
        .begin()
        .dst_set(&mut desc_set)?
        .uniform_buffers(
            0,
            0,
            &[vk::DescriptorBufferInfo {
                buffer: &uniform_buffer,
                offset: 0,
                range: u64::MAX,
            }],
        )?
        .end();
    let desc_set = Arc::new(desc_set);

    let pipeline_layout = device.create_pipeline_layout(
        Default::default(),
        vec![descriptor_set_layout.clone()],
        &[],
    )?;

    let pipeline =
        device.create_graphics_pipeline(&vk::GraphicsPipelineCreateInfo {
            stype: Default::default(),
            next: Default::default(),
            flags: Default::default(),
            stages: vk::Slice_::from_slice(&[
                vk::PipelineShaderStageCreateInfo::vertex(&vertex_shader),
                vk::PipelineShaderStageCreateInfo::fragment(&fragment_shader),
            ]),
            vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                vertex_binding_descriptions: vk::Slice_::from_slice(&[
                    vk::VertexInputBindingDescription {
                        binding: 0,
                        stride: size_of::<Vertex>() as u32,
                        input_rate: vk::VertexInputRate::VERTEX,
                    },
                ]),
                vertex_attribute_descriptions: vk::Slice::from_slice(&[
                    vk::VertexInputAttributeDescription {
                        location: 0,
                        binding: 0,
                        format: vk::Format::R32G32B32A32_SFLOAT,
                        offset: 0,
                    },
                    vk::VertexInputAttributeDescription {
                        location: 1,
                        binding: 0,
                        format: vk::Format::R32G32B32A32_SFLOAT,
                        offset: 16,
                    },
                ]),
                ..Default::default()
            },
            input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_STRIP,
                ..Default::default()
            },
            tessellation_state: None,
            viewport_state: &vk::PipelineViewportStateCreateInfo {
                viewports: vk::Slice_::from_slice(&[Default::default()]),
                scissors: vk::Slice::from_slice(&[vk::Rect2D {
                    offset: Default::default(),
                    extent: vk::Extent2D { width: 3840, height: 2160 },
                }]),
                ..Default::default()
            },
            rasterization_state: &Default::default(),
            multisample_state: &Default::default(),
            depth_stencil_state: None,
            color_blend_state: &Default::default(),
            dynamic_state: Some(&vk::PipelineDynamicStateCreateInfo {
                dynamic_states: vk::Slice_::from_slice(&[
                    vk::DynamicState::VIEWPORT,
                    //vk::DynamicState::SCISSOR,
                ]),
                ..Default::default()
            }),
            layout: pipeline_layout.borrow(),
            render_pass: render_pass.borrow(),
            subpass: 0,
            base_pipeline_handle: None,
            base_pipeline_index: 0,
        })?;

    let mut framebuffers = HashMap::new();

    let begin = Instant::now();

    type DrawSize = winit::dpi::PhysicalSize<u32>;
    let mut redraw = move |draw_size: DrawSize| -> anyhow::Result<()> {
        if draw_size != swapchain_size {
            swapchain_size = draw_size;
            framebuffers.clear();
            swapchain = Some(device.khr_swapchain().create(
                vk::CreateSwapchainFrom::OldSwapchain(
                    swapchain.take().unwrap(),
                ),
                vk::SwapchainCreateInfoKHR {
                    min_image_count: 3,
                    image_format: vk::Format::B8G8R8A8_SRGB,
                    image_extent: vk::Extent2D {
                        width: swapchain_size.width,
                        height: swapchain_size.height,
                    },
                    image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                        | vk::ImageUsageFlags::TRANSFER_DST,
                    ..Default::default()
                },
            )?);
        }

        let (img, _subopt) = swapchain
            .as_mut()
            .unwrap()
            .acquire_next_image(&mut acquire_sem, u64::MAX)?;

        let framebuffer = framebuffers
            .entry(img.clone())
            .or_insert_with(|| {
                let img_view = img.create_view(&vk::ImageViewCreateInfo {
                    format: vk::Format::B8G8R8A8_SRGB,
                    ..Default::default()
                })?;
                render_pass.create_framebuffer(
                    Default::default(),
                    vec![img_view],
                    vk::Extent3D {
                        width: swapchain_size.width,
                        height: swapchain_size.height,
                        depth: 1,
                    },
                )
            })
            .clone()?;

        let buf = cmd_pool.allocate()?;
        let mut rec = cmd_pool.begin(buf)?;
        let time = Instant::now().duration_since(begin);
        let blue = time.subsec_micros() as f32 / 1e6;

        let mut pass = rec.begin_render_pass(
            &render_pass,
            &framebuffer,
            vk::Rect2D {
                offset: Default::default(),
                extent: vk::Extent2D {
                    width: draw_size.width,
                    height: draw_size.height,
                },
            },
            &[vk::ClearValue {
                color: vk::ClearColorValue { f32: [0.1, 0.2, blue, 1.0] },
            }],
        );
        pass.set_viewport(&vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: draw_size.width as f32,
            height: draw_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        pass.bind_pipeline(vk::PipelineBindPoint::GRAPHICS, &pipeline);
        pass.bind_vertex_buffers(0, &[(&vertex_buffer, 0)])?;
        pass.bind_index_buffer(
            &vertex_buffer,
            index_offset as u64,
            vk::IndexType::UINT16,
        );
        pass.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            &pipeline_layout,
            0,
            &[&desc_set],
        )?;
        pass.draw_indexed(6, 1, 0, 0, 0);
        pass.end();

        let mut buf = rec.end()?;

        *bytemuck::from_bytes_mut(uniform_memory.slice_mut()) = UBO {
            model: Mat4::from_rotation_y(time.as_secs_f32() * 2.0),
            view: Mat4::look_at(
                Vec3::new(1., 1., 1.),
                Vec3::zero(),
                Vec3::new(0., 1., 0.),
            ),
            proj: ultraviolet::projection::perspective_infinite_z_vk(
                std::f32::consts::FRAC_PI_2,
                draw_size.width as f32 / draw_size.height as f32,
                0.1,
            ),
        };

        let pending_fence = queue.submit(
            &mut [vk::SubmitInfo {
                wait: &mut [(
                    &mut acquire_sem,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )],
                commands: &mut [&mut buf],
                signal: &mut [&mut present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.as_mut().unwrap().present(
            &mut queue,
            &img,
            &mut present_sem,
        )?;
        fence = Some(pending_fence.wait()?);
        drop(buf);
        cmd_pool.reset(Default::default())?;
        Ok(())
    };

    event_loop.run(move |event, _, control_flow| {
        use winit::event::{Event, WindowEvent};
        use winit::event_loop::ControlFlow;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
                if let Err(e) = redraw(window.inner_size()) {
                    println!("{:?}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        }
    })
}
