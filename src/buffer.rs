use crate::error::{Error, Result};
use crate::memory::DeviceMemory;
use crate::types::*;
use crate::vk::Device;

#[must_use = "Buffer is leaked if it is not bound to memory"]
#[derive(Debug)]
pub struct BufferWithoutMemory {
    handle: Handle<VkBuffer>,
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Buffer {
    handle: Handle<VkBuffer>,
    memory: Arc<DeviceMemory>,
}

impl Device {
    pub fn create_buffer(
        self: &Arc<Self>,
        info: &BufferCreateInfo<'_>,
    ) -> Result<BufferWithoutMemory> {
        let mut handle = None;
        unsafe {
            (self.fun.create_buffer)(self.borrow(), info, None, &mut handle)?;
        }
        Ok(BufferWithoutMemory {
            handle: handle.unwrap(),
            device: self.clone(),
        })
    }
}
impl DeviceMemory {
    pub fn bind_buffer_memory(
        self: &Arc<Self>,
        mut buffer: BufferWithoutMemory,
        offset: u64,
    ) -> Result<Arc<Buffer>> {
        let mem_req = buffer.memory_requirements();
        if !Arc::ptr_eq(&self.device, &buffer.device)
            || 1 << self.type_index() & mem_req.memory_type_bits == 0
            || offset & (mem_req.alignment - 1) != 0
        {
            return Err(Error::InvalidArgument);
        }
        unsafe {
            (self.device.fun.bind_buffer_memory)(
                self.device.borrow(),
                buffer.handle.borrow_mut(),
                self.borrow(),
                offset,
            )?;
        }
        Ok(Arc::new(Buffer { handle: buffer.handle, memory: self.clone() }))
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            (self.memory.device.fun.destroy_buffer)(
                self.memory.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Buffer {
    pub fn borrow(&self) -> Ref<VkBuffer> {
        self.handle.borrow()
    }
}
impl BufferWithoutMemory {
    pub fn borrow_mut(&mut self) -> Mut<VkBuffer> {
        self.handle.borrow_mut()
    }
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun.get_buffer_memory_requirements)(
                self.device.borrow(),
                self.handle.borrow(),
                &mut result,
            );
        }
        result
    }
}
