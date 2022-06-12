use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::memory::{DeviceMemory, MemoryPayload};
use crate::subobject::Subobject;
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
    _memory: Subobject<MemoryPayload>,
    pub(crate) device: Arc<Device>,
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
        &self,
        mut buffer: BufferWithoutMemory,
        offset: u64,
    ) -> ResultAndSelf<Arc<Buffer>, BufferWithoutMemory> {
        if !Arc::ptr_eq(&self.inner.device, &buffer.device)
            || !self.check(offset, buffer.memory_requirements())
        {
            return Err(ErrorAndSelf(Error::InvalidArgument, buffer));
        }

        if let Err(err) = unsafe {
            (self.inner.device.fun.bind_buffer_memory)(
                self.inner.device.borrow(),
                buffer.handle.borrow_mut(),
                self.borrow(),
                offset,
            )
        } {
            return Err(ErrorAndSelf(err.into(), buffer));
        }

        Ok(Arc::new(Buffer {
            handle: buffer.handle,
            _memory: Subobject::new(&self.inner),
            device: self.inner.device.clone(),
        }))
    }
}

impl Drop for Buffer {
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
