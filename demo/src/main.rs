use ember::vk;

fn main() -> anyhow::Result<()> {
    let inst = vk::create_instance(&vk::InstanceCreateInfo::S {
        next: None,
        flags: Default::default(),
        application_info: None,
        enabled_layer_names: vk::Slice::from(&[]),
        enabled_extension_names: vk::Slice::from(&[]),
    })?;
    println!("{:?}", inst);
    let phys = inst.enumerate_physical_devices()?;
    println!("{:?}", phys);
    println!("{:?}", phys[0].properties());
    Ok(())
}
