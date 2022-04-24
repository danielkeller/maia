use ember::vk;

fn pick_physical_device(
    mut phys: Vec<vk::PhysicalDevice>,
) -> vk::PhysicalDevice {
    for i in 0..phys.len() {
        if phys[i].properties().device_type
            == vk::PhysicalDeviceType::DiscreteGPU
        {
            return phys.swap_remove(i);
        }
    }
    for i in 0..phys.len() {
        if phys[i].properties().device_type
            == vk::PhysicalDeviceType::IntegratedGPU
        {
            return phys.swap_remove(i);
        }
    }
    phys.swap_remove(0)
}

fn pick_queue_family(phy: &vk::PhysicalDevice) -> anyhow::Result<u32> {
    let props = phy.queue_family_properties();
    for i in 0..props.len() {
        if !(props[i].queue_flags | vk::QueueFlags::GRAPHICS).is_empty() {
            return Ok(i.try_into().unwrap());
        }
    }
    anyhow::bail!("No graphics queue")
}

fn main() -> anyhow::Result<()> {
    use winit::event_loop::EventLoop;
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;

    let instance_exts = ember::window::required_instance_extensions(&window)?;
    let inst = vk::create_instance(&vk::InstanceCreateInfo::S {
        next: None,
        flags: Default::default(),
        application_info: None,
        enabled_layer_names: Default::default(),
        enabled_extension_names: instance_exts.into(),
    })?;
    println!("{:?}", inst);
    let phy = pick_physical_device(inst.enumerate_physical_devices()?);
    let queue_family = pick_queue_family(&phy)?;

    let device = phy.create_device(&vk::DeviceCreateInfo::S {
        next: None,
        flags: Default::default(),
        queue_create_infos: (&[vk::DeviceQueueCreateInfo::S {
            next: None,
            flags: Default::default(),
            queue_family_index: queue_family,
            queue_priorities: (&[1.0]).into(),
        }])
            .into(),
        enabled_layer_names: Default::default(),
        enabled_extension_names: Default::default(),
        enabled_features: None,
    })?;

    println!("{:?}", device);

    Ok(())
    // event_loop.run(move |event, _, control_flow| {
    //     use winit::event::{Event, WindowEvent};
    //     use winit::event_loop::ControlFlow;
    //     *control_flow = ControlFlow::Wait;
    //     match event {
    //         Event::WindowEvent {
    //             event: WindowEvent::CloseRequested, ..
    //         } => *control_flow = ControlFlow::Exit,
    //         _ => (),
    //     }
    // })
}
