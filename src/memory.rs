use crate::error::{Error, ErrorAndSelf, Result, ResultAndSelf};
use crate::subobject::Owner;
use crate::types::*;
use crate::vk::Device;

#[derive(Debug)]
pub(crate) struct MemoryPayload {
    handle: Handle<VkDeviceMemory>,
    pub(crate) device: Arc<Device>,
}

#[derive(Debug)]
pub struct DeviceMemory {
    pub(crate) inner: Owner<MemoryPayload>,
    allocation_size: u64,
    memory_type_index: u32,
}

impl Device {
    pub fn allocate_memory(
        self: &Arc<Self>,
        allocation_size: u64,
        memory_type_index: u32,
    ) -> Result<DeviceMemory> {
        let mem_types = self.physical_device().memory_properties();
        if memory_type_index >= mem_types.memory_types.len() {
            return Err(Error::OutOfBounds);
        }
        let mut handle = None;
        unsafe {
            (self.fun.allocate_memory)(
                self.handle(),
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
        Ok(DeviceMemory {
            allocation_size,
            memory_type_index,
            inner: Owner::new(MemoryPayload {
                handle: handle.unwrap(),
                device: self.clone(),
            }),
        })
    }
}

impl Drop for MemoryPayload {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.free_memory)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl DeviceMemory {
    pub fn handle(&self) -> Ref<VkDeviceMemory> {
        self.inner.handle.borrow()
    }
    pub fn handle_mut(&mut self) -> Mut<VkDeviceMemory> {
        self.inner.handle.borrow_mut()
    }
    pub fn check(&self, offset: u64, requirements: MemoryRequirements) -> bool {
        let (end, overflow) = offset.overflowing_add(requirements.size);
        (1 << self.memory_type_index) & requirements.memory_type_bits != 0
            && offset & (requirements.alignment - 1) == 0
            && !overflow
            && end <= self.allocation_size
    }
}

pub struct MappedMemory {
    memory: DeviceMemory,
    _offset: u64,
    size: usize,
    ptr: *mut u8,
}

impl DeviceMemory {
    pub fn map(
        mut self,
        offset: u64,
        size: usize,
    ) -> ResultAndSelf<MappedMemory, Self> {
        let (end, overflow) = offset.overflowing_add(size as u64);
        if overflow || end > self.allocation_size {
            return Err(ErrorAndSelf(Error::OutOfBounds, self));
        }
        let inner = &mut *self.inner;
        let mut ptr = std::ptr::null_mut();
        unsafe {
            if let Err(err) = (inner.device.fun.map_memory)(
                inner.device.handle(),
                inner.handle.borrow_mut(),
                offset,
                size as u64,
                Default::default(),
                &mut ptr,
            ) {
                return Err(ErrorAndSelf(err.into(), self));
            }
        }
        Ok(MappedMemory { memory: self, _offset: offset, size, ptr })
    }
    pub fn len(&self) -> u64 {
        self.allocation_size
    }
}

impl Drop for MappedMemory {
    fn drop(&mut self) {
        self.unmap_impl()
    }
}

impl MappedMemory {
    fn unmap_impl(&mut self) {
        let inner = &mut *self.memory.inner;
        unsafe {
            (inner.device.fun.unmap_memory)(
                inner.device.handle(),
                inner.handle.borrow_mut(),
            )
        }
    }

    pub fn unmap(mut self) -> DeviceMemory {
        self.unmap_impl();
        let no_drop = std::mem::ManuallyDrop::new(self);
        unsafe { std::ptr::addr_of!(no_drop.memory).read() }
    }

    pub fn memory(&self) -> &DeviceMemory {
        &self.memory
    }

    pub fn slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }

    pub fn slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
}
