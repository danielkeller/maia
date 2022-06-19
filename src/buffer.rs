use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::memory::{DeviceMemory, MemoryLifetime};
use crate::subobject::Subobject;
use crate::types::*;
use crate::vk::Device;

/// A buffer with no memory. Call [DeviceMemory::bind_buffer_memory] to bind
/// memory to the buffer.
#[derive(Debug)]
pub struct BufferWithoutMemory {
    handle: Handle<VkBuffer>,
    len: u64,
    device: Arc<Device>,
}

/// A buffer with memory attached to it.
#[derive(Debug)]
pub struct Buffer {
    inner: BufferWithoutMemory,
    _memory: Subobject<MemoryLifetime>,
}

impl Device {
    #[doc = crate::man_link!(vkCreateBuffer)]
    pub fn create_buffer(
        self: &Arc<Self>,
        info: &BufferCreateInfo<'_>,
    ) -> Result<BufferWithoutMemory> {
        let mut handle = None;
        unsafe {
            (self.fun.create_buffer)(self.handle(), info, None, &mut handle)?;
        }
        Ok(BufferWithoutMemory {
            handle: handle.unwrap(),
            len: info.size,
            device: self.clone(),
        })
    }
}
impl DeviceMemory {
    // TODO: Bulk bind
    #[doc = crate::man_link!(vkBindBufferMemory)]
    pub fn bind_buffer_memory(
        &self,
        buffer: BufferWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Buffer>, BufferWithoutMemory> {
        assert_eq!(self.device(), &*buffer.device);
        if !self.check(offset, buffer.memory_requirements()) {
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
            (self.device().fun.bind_buffer_memory)(
                self.device().handle(),
                inner.handle.borrow_mut(),
                self.handle(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), inner));
        }
        Ok(Arc::new(Buffer { inner, _memory: self.resource() }))
    }
}

impl Drop for BufferWithoutMemory {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.destroy_buffer)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl Buffer {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkBuffer> {
        self.inner.handle.borrow()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Device {
        &*self.inner.device
    }
    /// Returns the buffer length in bytes.
    pub fn len(&self) -> u64 {
        self.inner.len
    }
    /// Returns true if the offset and length are within the buffer's bounds.
    pub fn bounds_check(&self, offset: u64, len: u64) -> bool {
        self.len() >= offset && self.len() - offset >= len
    }
}

impl BufferWithoutMemory {
    /// Borrows the inner Vulkan handle.
    pub fn borrow_mut(&mut self) -> Mut<VkBuffer> {
        self.handle.borrow_mut()
    }
    #[doc = crate::man_link!(vkGetBufferMemoryRequirements)]
    pub fn memory_requirements(&self) -> MemoryRequirements {
        let mut result = Default::default();
        unsafe {
            (self.device.fun.get_buffer_memory_requirements)(
                self.device.handle(),
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
