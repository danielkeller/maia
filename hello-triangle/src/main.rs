use ember::vk;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: [f32; 4],
    color: [f32; 4],
}

const VERTEX_DATA: [Vertex; 3] = [
    Vertex { pos: [0.0, -0.7, 0., 1.], color: [0., 0., 1., 1.] },
    Vertex { pos: [-0.7, 0.7, 0., 1.], color: [1., 0., 0., 1.] },
    Vertex { pos: [0.7, 0.7, 0., 1.], color: [0., 1., 0., 1.] },
];

// Add extension required by MoltenVK, if available
fn required_instance_extensions() -> vk::Result<&'static [vk::Str<'static>]> {
    let exts = vk::instance_extension_properties()?;
    const FOR_MVK: vk::Str<'static> = vk::ext::GET_PHYSICAL_DEVICE_PROPERTIES2;
    if exts.iter().any(|e| e.extension_name == FOR_MVK) {
        Ok(&[FOR_MVK])
    } else {
        Ok(&[])
    }
}

// Add extension required by MoltenVK, if available
fn required_device_extensions(
    phy: &vk::PhysicalDevice,
) -> vk::Result<&'static [vk::Str<'static>]> {
    let exts = phy.device_extension_properties()?;
    const FOR_MVK: vk::Str<'static> = vk::ext::PORTABILITY_SUBSET;
    if exts.iter().any(|e| e.extension_name == FOR_MVK) {
        Ok(&[FOR_MVK, vk::ext::SWAPCHAIN])
    } else {
        Ok(&[vk::ext::SWAPCHAIN])
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
) -> vk::Result<u32> {
    for (num, props) in phy.queue_family_properties().iter().enumerate() {
        if !(props.queue_flags & vk::QueueFlags::GRAPHICS).is_empty()
            && surf.support(phy, num as u32)?
        {
            return Ok(num as u32);
        }
    }
    panic!("No usable queue!")
}

fn memory_type(phy: &vk::PhysicalDevice) -> u32 {
    let desired = vk::MemoryPropertyFlags::HOST_VISIBLE
        | vk::MemoryPropertyFlags::HOST_COHERENT;
    for (num, props) in phy.memory_properties().memory_types.iter().enumerate()
    {
        if props.property_flags & desired == desired {
            return num as u32;
        }
    }
    panic!("No host visible memory!")
}

fn main() -> vk::Result<()> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

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
            format: vk::Format::B8G8R8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR,
        }
    }) {
        panic!("Desired surface format not found");
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

    let window_size = window.inner_size();
    let window_extent = vk::Extent2D {
        width: window_size.width,
        height: window_size.height,
    };
    let mut swapchain = device.khr_swapchain().create(
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: window_extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ..Default::default()
        },
    )?;
    let mut framebuffers = std::collections::HashMap::new();

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

    let vertex_shader =
        device.create_shader_module(inline_spirv::inline_spirv!(
            r#" #version 450
                layout(location = 0) in vec4 i_position;
                layout(location = 1) in vec4 i_color;
                layout(location = 0) out vec4 o_color;
                void main() {
                    gl_Position = i_position;
                    o_color = i_color;
                } "#,
            glsl,
            vert
        ))?;
    let fragment_shader =
        device.create_shader_module(inline_spirv::inline_spirv!(
            r#" #version 450
                layout(location = 0) in vec4 i_color;
                layout(location = 0) out vec4 o_Color;
                void main() { o_Color = i_color;} "#,
            glsl,
            frag
        ))?;

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
                        stride: std::mem::size_of::<Vertex>() as u32,
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
            viewport_state: &Default::default(),
            rasterization_state: &Default::default(),
            multisample_state: &Default::default(),
            depth_stencil_state: None,
            color_blend_state: &Default::default(),
            dynamic_state: Some(&vk::PipelineDynamicStateCreateInfo {
                dynamic_states: vk::slice(&[
                    vk::DynamicState::VIEWPORT,
                    vk::DynamicState::SCISSOR,
                ]),
                ..Default::default()
            }),
            layout: &device.create_pipeline_layout(
                Default::default(),
                vec![],
                vec![],
            )?,
            render_pass: &render_pass,
            subpass: 0,
            cache: None,
        })?;

    let vertex_size = std::mem::size_of_val(&VERTEX_DATA) as u64;
    let vertex_buffer = device.create_buffer(&vk::BufferCreateInfo {
        size: vertex_size,
        usage: vk::BufferUsageFlags::VERTEX_BUFFER,
        ..Default::default()
    })?;
    let memory = device.allocate_memory(
        vertex_buffer.memory_requirements().size,
        memory_type(&phy),
    )?;
    let vertex_buffer = memory.bind_buffer_memory(vertex_buffer, 0)?;
    let mut mapped = memory.map(0, std::mem::size_of_val(&VERTEX_DATA))?;
    mapped.slice_mut().copy_from_slice(bytemuck::bytes_of(&VERTEX_DATA));

    let mut cmd_pool = device.create_command_pool(queue_family)?;

    let mut queue = device.queue(0, 0)?;

    let mut acquire_sem = device.create_semaphore()?;
    let mut fence = Some(device.create_fence()?);

    let mut redraw = move || -> vk::Result<()> {
        let (img, _subopt) =
            swapchain.acquire_next_image(&mut acquire_sem, u64::MAX)?;

        if !framebuffers.contains_key(&img) {
            let img_view = img.create_view(&vk::ImageViewCreateInfo {
                format: vk::Format::B8G8R8A8_SRGB,
                ..Default::default()
            })?;
            let fb = render_pass.create_framebuffer(
                Default::default(),
                vec![img_view],
                window_extent.into(),
            )?;
            let sem = device.create_semaphore()?;
            framebuffers.insert(img.clone(), (fb, sem));
        }
        let (framebuffer, present_sem) = framebuffers.get_mut(&img).unwrap();

        let cmd_buf = cmd_pool.allocate()?;
        let mut pass = cmd_pool.begin(cmd_buf)?.begin_render_pass(
            &render_pass,
            &framebuffer,
            &vk::Rect2D { extent: window_extent, ..Default::default() },
            &[vk::ClearValue {
                color: vk::ClearColorValue { f32: [0.1, 0.2, 0.3, 1.0] },
            }],
        )?;
        pass.bind_pipeline(&pipeline);
        pass.set_viewport(&vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: window_extent.width as f32,
            height: window_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        });
        pass.set_scissor(&vk::Rect2D {
            extent: window_extent,
            ..Default::default()
        });
        pass.bind_vertex_buffers(0, &[(&vertex_buffer, 0)])?;
        pass.draw(3, 1, 0, 0)?;
        let mut cmd_buf = pass.end()?.end()?;

        let pending_fence = queue.submit(
            &mut [vk::SubmitInfo {
                wait: &mut [(
                    &mut acquire_sem,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )],
                commands: &mut [&mut cmd_buf],
                signal: &mut [present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.present(&mut queue, &img, present_sem)?;
        fence = Some(pending_fence.wait()?);
        drop(cmd_buf);
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
                if let Err(e) = redraw() {
                    println!("{:?}", e);
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        }
    })
}
