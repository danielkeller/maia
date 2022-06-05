use crate::error::{Error, Result};
use crate::types::*;
use crate::vk::Device;

#[derive(Debug)]
pub struct DeviceMemory {
    handle: Handle<VkDeviceMemory>,
    memory_type_index: u32,
    pub(crate) device: Arc<Device>,
}

impl Device {
    pub fn allocate_memory(
        self: &Arc<Self>,
        allocation_size: u64,
        memory_type_index: u32,
    ) -> Result<Arc<DeviceMemory>> {
        let mem_types = self.physical_device().memory_properties();
        if memory_type_index >= mem_types.memory_types.len() {
            return Err(Error::InvalidArgument);
        }
        let mut handle = None;
        unsafe {
            (self.fun.allocate_memory)(
                self.borrow(),
                &MemoryAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    allocation_size,
                    memory_type_index,
                },
                None,
                &mut handle,
            )?;
        }
        Ok(Arc::new(DeviceMemory {
            handle: handle.unwrap(),
            memory_type_index,
            device: self.clone(),
        }))
    }
}

impl Drop for DeviceMemory {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.free_memory)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DeviceMemory {
    pub fn borrow(&self) -> Ref<VkDeviceMemory> {
        self.handle.borrow()
    }
    pub fn type_index(&self) -> u32 {
        self.memory_type_index
    }
}
