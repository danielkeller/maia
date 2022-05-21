use std::time::Instant;

use ember::vk;

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
        queue_create_infos: (&[vk::DeviceQueueCreateInfo {
            queue_family_index: queue_family,
            queue_priorities: (&[1.0]).into(),
            ..Default::default()
        }])
            .into(),
        enabled_extension_names: device_extensions.into(),
        ..Default::default()
    })?;
    let mut queue = device.queue(0, 0)?;

    let mut swapchain = device.khr_swapchain().create(
        vk::CreateSwapchainFrom::Surface(surf),
        vk::SwapchainCreateInfoKHR {
            min_image_count: 3,
            image_format: vk::Format::B8G8R8A8_SRGB,
            image_extent: vk::Extent2D { width: 3840, height: 2160 },
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST,
            ..Default::default()
        },
    )?;

    let mut cmd_pool = device.create_command_pool(queue_family)?;

    let mut acquire_sem = device.create_semaphore()?;
    let mut present_sem = device.create_semaphore()?;
    let mut fence = Some(device.create_fence()?);

    let begin = Instant::now();

    let mut redraw = move || -> anyhow::Result<()> {
        let (img, _subopt) =
            swapchain.acquire_next_image(&mut acquire_sem, u64::MAX)?;

        cmd_pool.reset(Default::default())?;
        let mut buf = cmd_pool.allocate()?;
        let mut rec = cmd_pool.begin(&mut buf)?;
        let blue =
            Instant::now().duration_since(begin).subsec_micros() as f32 / 1e6;
        rec.clear_color_image(
            &img,
            vk::ImageLayout::GENERAL,
            vk::ClearColor::F32([0.1, 0.2, blue, 1.0]),
            &[Default::default()],
        )?;
        rec.end()?;
        let pending_fence = queue.submit(
            &mut [vk::SubmitInfo {
                wait: &[(&acquire_sem, vk::PipelineStageFlags::TOP_OF_PIPE)],
                commands: &mut [&mut buf],
                signal: &[&present_sem],
            }],
            fence.take().unwrap(),
        )?;
        swapchain.present(&mut queue, &img, &mut present_sem)?;
        fence = Some(pending_fence.wait()?);
        Ok(())
    };

    // Ok(())
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
