use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::memory::{DeviceMemory, MemoryPayload};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

#[derive(Debug)]
pub struct BufferWithoutMemory {
    handle: Handle<VkBuffer>,
    len: u64,
    device: Arc<Device>,
}

#[derive(Debug)]
pub struct Buffer {
    inner: BufferWithoutMemory,
    _memory: Subobject<MemoryPayload>,
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
            len: info.size,
            device: self.clone(),
        })
    }
}
impl DeviceMemory {
    pub fn bind_buffer_memory(
        &self,
        buffer: BufferWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Buffer>, BufferWithoutMemory> {
        if !Arc::ptr_eq(&self.inner.device, &buffer.device)
            || !self.check(offset, buffer.memory_requirements())
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }
        self.bind_buffer_impl(buffer, offset)
    }

    fn bind_buffer_impl(
        &self,
        mut inner: BufferWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Buffer>, BufferWithoutMemory> {
        if let Err(err) = unsafe {
            (self.inner.device.fun.bind_buffer_memory)(
                self.inner.device.borrow(),
                inner.handle.borrow_mut(),
                self.borrow(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), inner));
        }
        Ok(Arc::new(Buffer { inner, _memory: Subobject::new(&self.inner) }))
    }
}

impl Drop for BufferWithoutMemory {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_buffer)(
                self.device.borrow(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Buffer {
    pub fn borrow(&self) -> Ref<VkBuffer> {
        self.inner.handle.borrow()
    }
    pub fn device(&self) -> &Device {
        &*self.inner.device
    }
    pub fn len(&self) -> u64 {
        self.inner.len
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
    /// Allocate a single piece of memory for the buffer and bind it.
    pub fn allocate_memory(
        self,
        memory_type_index: u32,
    ) -> ResultAndSelf<Arc<Buffer>, Self> {
        let mem_req = self.memory_requirements();
        if (1 << memory_type_index) & mem_req.memory_type_bits == 0 {
            return Err(ErrorAndSelf(Error::InvalidArgument, self));
        }
        let memory = match self
            .device
            .allocate_memory(mem_req.size, memory_type_index)
        {
            Ok(memory) => memory,
            Err(err) => return Err(ErrorAndSelf(err.into(), self)),
        };
        // Don't need to check requirements
        memory.bind_buffer_impl(self, 0)
    }
}
