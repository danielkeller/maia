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
    uv: [f32; 2],
}

const VERTEX_DATA: [Vertex; 4] = [
    Vertex { pos: [-0.7, -0.7, 0.0, 1.0], uv: [1.0, 1.0] },
    Vertex { pos: [-0.7, 0.7, 0.0, 1.0], uv: [1.0, 0.0] },
    Vertex { pos: [0.7, -0.7, 0.0, 1.0], uv: [0.0, 1.0] },
    Vertex { pos: [0.7, 0.7, 0.0, 1.0], uv: [0.0, 0.0] },
];
const INDEX_DATA: [u16; 6] = [0, 1, 2, 2, 1, 3];

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct MVP {
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

fn pick_physical_device(phys: &[vk::PhysicalDevice]) -> vk::PhysicalDevice {
    let discr = vk::PhysicalDeviceType::DISCRETE_GPU;
    let int = vk::PhysicalDeviceType::INTEGRATED_GPU;
    phys.iter()
        .find(|p| p.properties().device_type == discr)
        .or_else(|| phys.iter().find(|p| p.properties().device_type == int))
        .unwrap_or(&phys[0])
        .clone()
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

fn upload_data(
    device: &Arc<vk::Device>,
    queue: &mut vk::Queue,
    cmd_pool: &mut vk::CommandPool,
    src: &[u8],
    dst: &Arc<vk::Buffer>,
    dst_stage_mask: vk::PipelineStageFlags,
    dst_access_mask: vk::AccessFlags,
) -> anyhow::Result<()> {
    let staging_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: src.len() as u64,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        ..Default::default()
    })?;
    let mem_size = staging_buffer.memory_requirements().size;
    let host_mem = memory_type(
        device.physical_device(),
        vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let memory = device.allocate_memory(mem_size as u64, host_mem)?;
    let staging_buffer = memory.bind_buffer_memory(staging_buffer, 0)?;
    let mut memory = memory.map(0, src.len())?;
    memory.slice_mut().copy_from_slice(src);

    let transfer = cmd_pool.allocate()?;
    let mut rec = cmd_pool.begin(transfer)?;
    rec.copy_buffer(
        &staging_buffer,
        dst,
        &[vk::BufferCopy { size: mem_size, ..Default::default() }],
    )?;
    rec.memory_barrier(
        vk::PipelineStageFlags::TRANSFER,
        dst_stage_mask,
        vk::AccessFlags::TRANSFER_WRITE,
        dst_access_mask,
    );
    let mut transfer = rec.end()?;
    let fence = device.create_fence()?;
    let pending_fence = queue.submit(
        &mut [vk::SubmitInfo {
            commands: &mut [&mut transfer],
            ..Default::default()
        }],
        fence,
    )?;
    pending_fence.wait()?;
    Ok(())
}

fn upload_image(
    device: &Arc<vk::Device>,
    queue: &mut vk::Queue,
    image: &Arc<vk::Image>,
    cmd_pool: &mut vk::CommandPool,
) -> anyhow::Result<()> {
    let image_data = image::io::Reader::open("assets/texture.jpg")?.decode()?;
    let image_data =
        image_data.as_rgb8().ok_or(anyhow::anyhow!("Wrong image type"))?;

    let staging_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: (image_data.len() / 3 * 4) as u64,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        ..Default::default()
    })?;
    let mem_size = staging_buffer.memory_requirements().size;
    let host_mem = memory_type(
        device.physical_device(),
        vk::MemoryPropertyFlags::HOST_VISIBLE
            | vk::MemoryPropertyFlags::HOST_COHERENT,
    );
    let memory = device.allocate_memory(mem_size, host_mem)?;
    let staging_buffer = memory.bind_buffer_memory(staging_buffer, 0)?;
    let mut memory = memory.map(0, mem_size as usize)?;
    for (dst, src) in
        memory.slice_mut().chunks_exact_mut(4).zip(image_data.chunks_exact(3))
    {
        dst[..3].copy_from_slice(src)
    }
    memory.unmap();

    let transfer = cmd_pool.allocate()?;
    let mut rec = cmd_pool.begin(transfer)?;
    rec.image_barrier(
        image,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::AccessFlags::default(),
        vk::AccessFlags::TRANSFER_WRITE,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    rec.copy_buffer_to_image(
        &staging_buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[vk::BufferImageCopy {
            image_extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
            ..Default::default()
        }],
    )?;
    rec.image_barrier(
        image,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::AccessFlags::TRANSFER_WRITE,
        vk::AccessFlags::MEMORY_READ,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let mut transfer = rec.end()?;
    let fence = device.create_fence()?;
    let pending_fence = queue.submit(
        &mut [vk::SubmitInfo {
            commands: &mut [&mut transfer],
            ..Default::default()
        }],
        fence,
    )?;
    pending_fence.wait()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;

    let mut instance_exts = vec![];
    instance_exts
        .extend(ember::window::required_instance_extensions(&window)?.iter());
    instance_exts.extend(required_instance_extensions()?.iter());
    let inst = vk::Instance::new(&vk::InstanceCreateInfo {
        enabled_extension_names: instance_exts.as_slice().into(),
        ..Default::default()
    })?;

    let surf = ember::window::create_surface(&inst, &window)?;

    let phy = pick_physical_device(&inst.enumerate_physical_devices()?);
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
        queue_create_infos: vk::slice(&[vk::DeviceQueueCreateInfo {
            queue_family_index: queue_family,
            queue_priorities: vk::slice(&[1.0]),
            ..Default::default()
        }]),
        enabled_extension_names: device_extensions.into(),
        ..Default::default()
    })?;
    let mut queue = device.queue(0, 0)?;

    let mut acquire_sem = device.create_semaphore()?;
    let mut fence = Some(device.create_fence()?);

    let window_size = window.inner_size();
    let mut swapchain_size = vk::Extent2D {
        width: window_size.width,
        height: window_size.height,
    };
    let mut swapchain = Some(device.khr_swapchain().create(
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: swapchain_size,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?);

    let mut cmd_pool = device.create_command_pool(queue_family)?;

    let vertex_size = std::mem::size_of_val(&VERTEX_DATA);
    let index_size = std::mem::size_of_val(&INDEX_DATA);

    let device_mem = memory_type(
        &device.physical_device(),
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    let vertex_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: vertex_size as u64,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
        ..Default::default()
    })?;
    let vertex_buffer = vertex_buffer.allocate_memory(device_mem)?;
    upload_data(
        &device,
        &mut queue,
        &mut cmd_pool,
        bytemuck::bytes_of(&VERTEX_DATA),
        &vertex_buffer,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::VERTEX_ATTRIBUTE_READ,
    )?;

    let index_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: index_size as u64,
        usage: vk::BufferUsageFlags::INDEX_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST,
        ..Default::default()
    })?;
    let index_buffer = index_buffer.allocate_memory(device_mem)?;
    upload_data(
        &device,
        &mut queue,
        &mut cmd_pool,
        bytemuck::bytes_of(&INDEX_DATA),
        &index_buffer,
        vk::PipelineStageFlags::VERTEX_INPUT,
        vk::AccessFlags::INDEX_READ,
    )?;

    let image = device.create_image(&vk::ImageCreateInfo {
        format: vk::Format::R8G8B8A8_SRGB,
        extent: vk::Extent3D { width: 512, height: 512, depth: 1 },
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        ..Default::default()
    })?;
    let image = image.allocate_memory(device_mem)?;
    upload_image(&device, &mut queue, &image, &mut cmd_pool)?;
    let image_view = image.create_view(&vk::ImageViewCreateInfo {
        format: vk::Format::R8G8B8A8_SRGB,
        ..Default::default()
    })?;

    let render_pass = device.create_render_pass(&vk::RenderPassCreateInfo {
        attachments: vk::slice(&[vk::AttachmentDescription {
            format: vk::Format::B8G8R8A8_SRGB,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        }]),
        subpasses: vk::slice(&[vk::SubpassDescription {
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
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            immutable_samplers: vec![
                device.create_sampler(&Default::default())?
            ],
        },
    ])?;
    let mut descriptor_pool = device.create_descriptor_pool(
        1,
        &[vk::DescriptorPoolSize {
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }],
    )?;
    let mut desc_set = descriptor_pool.allocate(&descriptor_set_layout)?;

    let mut update = device.create_descriptor_set_update_builder();
    update
        .begin()
        .dst_set(&mut desc_set)
        .combined_image_samplers(
            0,
            0,
            &[(&image_view, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)],
        )?
        .end();
    let desc_set = Arc::new(desc_set);

    let pipeline_layout = device.create_pipeline_layout(
        Default::default(),
        vec![descriptor_set_layout.clone()],
        vec![vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: std::mem::size_of::<MVP>() as u32,
        }],
    )?;

    let pipeline =
        device.create_graphics_pipeline(&vk::GraphicsPipelineCreateInfo {
            stages: &[
                vk::PipelineShaderStageCreateInfo::vertex(&vertex_shader),
                vk::PipelineShaderStageCreateInfo::fragment(&fragment_shader),
            ],
            vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                vertex_binding_descriptions: vk::slice(&[
                    vk::VertexInputBindingDescription {
                        binding: 0,
                        stride: size_of::<Vertex>() as u32,
                        input_rate: vk::VertexInputRate::VERTEX,
                    },
                ]),
                vertex_attribute_descriptions: vk::slice(&[
                    vk::VertexInputAttributeDescription {
                        location: 0,
                        binding: 0,
                        format: vk::Format::R32G32B32A32_SFLOAT,
                        offset: 0,
                    },
                    vk::VertexInputAttributeDescription {
                        location: 1,
                        binding: 0,
                        format: vk::Format::R32G32_SFLOAT,
                        offset: 16,
                    },
                ]),
                ..Default::default()
            },
            input_assembly_state: &vk::PipelineInputAssemblyStateCreateInfo {
                topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                ..Default::default()
            },
            tessellation_state: None,
            viewport_state: &vk::PipelineViewportStateCreateInfo {
                viewports: vk::slice(&[Default::default()]),
                scissors: vk::slice(&[vk::Rect2D {
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
                dynamic_states: vk::slice(&[vk::DynamicState::VIEWPORT]),
                ..Default::default()
            }),
            layout: &pipeline_layout,
            render_pass: &render_pass,
            subpass: 0,
            cache: None,
        })?;

    let mut framebuffers = HashMap::new();

    let begin = Instant::now();

    let mut redraw = move |draw_size: vk::Extent2D| -> anyhow::Result<()> {
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
                    image_extent: swapchain_size,
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

        if !framebuffers.contains_key(&img) {
            let img_view = img.create_view(&vk::ImageViewCreateInfo {
                format: vk::Format::B8G8R8A8_SRGB,
                ..Default::default()
            })?;
            let fb = render_pass.create_framebuffer(
                Default::default(),
                vec![img_view],
                swapchain_size.into(),
            )?;
            let sem = device.create_semaphore()?;
            framebuffers.insert(img.clone(), (fb, sem));
        }
        let (framebuffer, present_sem) = framebuffers.get_mut(&img).unwrap();

        let time = Instant::now().duration_since(begin);

        let mvp = MVP {
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

        let subpass = cmd_pool.allocate_secondary()?;
        let mut subpass = cmd_pool.begin_secondary(subpass, &render_pass, 0)?;
        subpass.set_viewport(&vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: draw_size.width as f32,
            height: draw_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        subpass.bind_pipeline(&pipeline);
        subpass.bind_vertex_buffers(0, &[(&vertex_buffer, 0)])?;
        subpass.bind_index_buffer(&index_buffer, 0, vk::IndexType::UINT16);
        subpass.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            &pipeline_layout,
            0,
            &[&desc_set],
            &[],
        )?;
        subpass.push_constants(
            &pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&mvp),
        )?;
        subpass.draw_indexed(6, 1, 0, 0, 0)?;
        let mut subpass = subpass.end()?;

        let cmd = cmd_pool.allocate()?;
        let mut pass = cmd_pool.begin(cmd)?.begin_render_pass_secondary(
            &render_pass,
            &framebuffer,
            &vk::Rect2D {
                offset: Default::default(),
                extent: vk::Extent2D {
                    width: draw_size.width,
                    height: draw_size.height,
                },
            },
            &[vk::ClearValue {
                color: vk::ClearColorValue { f32: [0.1, 0.2, 0.3, 1.0] },
            }],
        )?;
        pass.execute_commands(&mut [&mut subpass])?;
        let mut buf = pass.end()?.end()?;

        let pending_fence = queue.submit(
            &mut [vk::SubmitInfo {
                wait: &mut [(
                    &mut acquire_sem,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )],
                commands: &mut [&mut buf],
                signal: &mut [present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.as_mut().unwrap().present(&mut queue, &img, present_sem)?;
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
                let window_size = window.inner_size();
                let window_size = vk::Extent2D {
                    width: window_size.width,
                    height: window_size.height,
                };
                if let Err(e) = redraw(window_size) {
                    println!("{:?}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        }
    })
}
