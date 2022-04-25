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

fn required_device_extensions(
    phy: &vk::PhysicalDevice,
) -> anyhow::Result<&'static [vk::Str<'static>]> {
    let exts = phy.device_extension_properties()?;
    if exts.iter().any(|e| e.extension_name == vk::ext::PORTABILITY_SUBSET) {
        Ok(std::slice::from_ref(&vk::ext::PORTABILITY_SUBSET))
    } else {
        Ok(&[])
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
    let inst = vk::create_instance(&vk::InstanceCreateInfo::S {
        next: None,
        flags: Default::default(),
        application_info: None,
        enabled_layer_names: Default::default(),
        enabled_extension_names: instance_exts.as_slice().into(),
    })?;
    println!("{:?}", inst);
    let phy = pick_physical_device(inst.enumerate_physical_devices()?);
    println!("{:?}", phy);
    let queue_family = pick_queue_family(&phy)?;

    let device_extensions = required_device_extensions(&phy)?;
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
        enabled_extension_names: device_extensions.into(),
        enabled_features: None,
    })?;

    println!("{:?}", device);
    println!("{:?}", device.queue(0, 0)?);

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
