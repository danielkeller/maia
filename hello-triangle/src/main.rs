use maia::vk;

fn main() -> vk::Result<()> {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    let mut instance_exts = vec![];
    // Get the instance extensions needed to create a surface for the window
    instance_exts
        .extend(maia::window::required_instance_extensions(&window)?.iter());
    // Get the instance extension needed by MoltenVK, if neccesary
    instance_exts.extend(required_instance_extensions()?.iter());
    // Create the instance
    let inst = vk::Instance::new(&vk::InstanceCreateInfo {
        enabled_extension_names: vk::slice(&instance_exts),
        ..Default::default()
    })?;

    // Create a surface with the appropriate platform extension
    let surf = maia::window::create_surface(&inst, &window)?;

    // Pick a suitable physical device
    let phy = pick_physical_device(&inst.enumerate_physical_devices()?);
    // Pick a queue family that supports presenting to the surface
    let queue_family = pick_queue_family(&phy, &surf, &window)?;
    // Make sure the surface format we want is supported
    assert!(surf.surface_formats(&phy)?.iter().any(|f| {
        f == &vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_SRGB,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR,
        }
    }));

    // We need the swapchain extension, plus the extension needed by MoltenVK
    let device_extensions = required_device_extensions(&phy)?;
    // Create the virtual device
    let (device, mut queues) = vk::Device::new(
        &phy,
        &vk::DeviceCreateInfo {
            // Create one queue
            queue_create_infos: vk::slice(&[vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family,
                queue_priorities: vk::slice(&[1.0]),
                ..Default::default()
            }]),
            enabled_extension_names: vk::slice(device_extensions),
            ..Default::default()
        },
    )?;

    // Create the swapchain
    let window_size = window.inner_size();
    let window_extent = vk::Extent2D {
        width: window_size.width,
        height: window_size.height,
    };
    let mut swapchain = vk::ext::SwapchainKHR::new(
        &device,
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: window_extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            ..Default::default()
        },
    )?;
    // We need one framebuffer per image, so put them in a hashmap
    let mut framebuffers = std::collections::HashMap::new();

    // Create the render pass
    let render_pass = vk::RenderPass::new(
        &device,
        &vk::RenderPassCreateInfo {
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
        },
    )?;

    // Create the shader modules and graphics pipeline
    let vertex_shader = vk::ShaderModule::new(
        &device,
        inline_spirv::inline_spirv!(
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
        ),
    )?;
    let fragment_shader = vk::ShaderModule::new(
        &device,
        inline_spirv::inline_spirv!(
            r#" #version 450
                layout(location = 0) in vec4 i_color;
                layout(location = 0) out vec4 o_Color;
                void main() { o_Color = i_color;} "#,
            glsl,
            frag
        ),
    )?;

    let pipeline =
        vk::Pipeline::new_graphics(&vk::GraphicsPipelineCreateInfo {
            stages: &[
                vk::PipelineShaderStageCreateInfo::vertex(&vertex_shader),
                vk::PipelineShaderStageCreateInfo::fragment(&fragment_shader),
            ],
            vertex_input_state: &vk::PipelineVertexInputStateCreateInfo {
                vertex_binding_descriptions: vk::slice(&[
                    vk::VertexInputBindingDescription {
                        binding: 0,
                        stride: std::mem::size_of_val(&VERTEX_DATA[0]) as u32,
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
            // It's good practice to specify the viewport as dynamic state, so
            // we don't need to recompile every pipeline when the window is
            // resized
            dynamic_state: Some(&vk::PipelineDynamicStateCreateInfo {
                dynamic_states: vk::slice(&[
                    vk::DynamicState::VIEWPORT,
                    vk::DynamicState::SCISSOR,
                ]),
                ..Default::default()
            }),
            layout: &vk::PipelineLayout::new(
                &device,
                Default::default(),
                vec![],
                vec![],
            )?,
            render_pass: &render_pass,
            subpass: 0,
            cache: None,
        })?;

    // Create the vertex buffer and fill it with data
    let vertex_buffer = vk::BufferWithoutMemory::new(
        &device,
        &vk::BufferCreateInfo {
            size: std::mem::size_of_val(&VERTEX_DATA) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            ..Default::default()
        },
    )?;
    let memory = vk::DeviceMemory::new(
        &device,
        vertex_buffer.memory_requirements().size,
        memory_type(&phy),
    )?;
    let vertex_buffer = vk::Buffer::new(vertex_buffer, &memory, 0)?;
    let mut mapped = memory.map(0, std::mem::size_of_val(&VERTEX_DATA))?;
    mapped.slice_mut().copy_from_slice(bytemuck::bytes_of(&VERTEX_DATA));

    // Create the remaining required objects
    let mut cmd_pool = vk::CommandPool::new(&device, queue_family)?;
    // Create the command buffer
    let mut cmd_buf = Some(cmd_pool.allocate()?);
    let mut queue = queues.remove(0).remove(0);
    // Since we wait between frames we only need one acquire semaphore
    let mut acquire_sem = vk::Semaphore::new(&device)?;
    let mut fence = Some(vk::Fence::new(&device)?);

    let mut redraw = move || -> vk::Result<()> {
        let (img, _subopt) =
            swapchain.acquire_next_image(&mut acquire_sem, u64::MAX)?;

        // We want one framebuffer and present semaphore per image
        if !framebuffers.contains_key(&img) {
            // Create them if we haven't already
            let img_view = vk::ImageView::new(
                &img,
                &vk::ImageViewCreateInfo {
                    format: vk::Format::B8G8R8A8_SRGB,
                    ..Default::default()
                },
            )?;
            let fb = vk::Framebuffer::new(
                &render_pass,
                Default::default(),
                vec![img_view],
                window_extent.into(),
            )?;
            let sem = vk::Semaphore::new(&device)?;
            framebuffers.insert(img.clone(), (fb, sem));
        }
        let (framebuffer, present_sem) = framebuffers.get_mut(&img).unwrap();

        // Command buffer recoding uses a builder pattern
        let mut pass = cmd_pool
            .begin(cmd_buf.take().unwrap())?
            // The render pass is recorded on a builder that wraps the main one
            .begin_render_pass(
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
        // End the render pass and extract the command buffer from the builder
        cmd_buf = Some(pass.end()?.end()?);

        // Submit the command buffer
        let pending_fence = queue.submit_with_fence(
            &mut [vk::SubmitInfo {
                wait: &mut [(
                    &mut acquire_sem,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                )],
                commands: &mut [cmd_buf.as_mut().unwrap()],
                signal: &mut [present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.present(&mut queue, &img, present_sem)?;
        // Wait for the execution to finish. Otherwise resetting the command
        // pool will return an error.
        fence = Some(pending_fence.wait()?);
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
            _ => (),
        }
    })
}

const VERTEX_DATA: [[[f32; 4]; 2]; 3] = [
    [[0.0, -0.7, 0., 1.], [0., 0., 1., 1.]],
    [[-0.7, 0.7, 0., 1.], [1., 0., 0., 1.]],
    [[0.7, 0.7, 0., 1.], [0., 1., 0., 1.]],
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

// Pick an appropriate physical device
fn pick_physical_device(phys: &[vk::PhysicalDevice]) -> vk::PhysicalDevice {
    let discr = vk::PhysicalDeviceType::DISCRETE_GPU;
    let int = vk::PhysicalDeviceType::INTEGRATED_GPU;
    phys.iter()
        .find(|p| p.properties().device_type == discr)
        .or_else(|| phys.iter().find(|p| p.properties().device_type == int))
        .unwrap_or(&phys[0])
        .clone()
}

// Pick an appropriate queue family
fn pick_queue_family(
    phy: &vk::PhysicalDevice,
    surf: &vk::ext::SurfaceKHR,
    window: &winit::window::Window,
) -> vk::Result<u32> {
    for (num, props) in phy.queue_family_properties().iter().enumerate() {
        if !(props.queue_flags & vk::QueueFlags::GRAPHICS).is_empty()
            && surf.support(phy, num as u32)?
            && maia::window::presentation_support(phy, num as u32, window)
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
