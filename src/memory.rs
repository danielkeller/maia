// Copyright 2022 Google LLC

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::error::{Error, Result, ResultAnd};
use crate::subobject::{Owner, Subobject};
use crate::types::*;
use crate::vk::Device;

#[derive(Debug)]
pub struct MemoryLifetime {
    handle: Handle<VkDeviceMemory>,
    device: Arc<Device>,
}

/// A piece of
#[doc = crate::spec_link!("device memory", "11", "memory-device")]
#[derive(Debug)]
pub struct DeviceMemory {
    inner: Owner<MemoryLifetime>,
    allocation_size: u64,
    memory_type_index: u32,
}

impl DeviceMemory {
    /// Returns [`ErrorKind::OutOfBounds`] if no memory type exists with the given
    /// index.
    #[doc = crate::man_link!(vkAllocateMemory)]
    pub fn new(
        device: &Arc<Device>, allocation_size: u64, memory_type_index: u32,
    ) -> Result<Self> {
        let mem_types = device.physical_device().memory_properties();
        if memory_type_index >= mem_types.memory_types.len() {
            Err(Error::out_of_bounds(format!(
                "Memory type {memory_type_index} does not exist"
            )))?
        }
        if allocation_size > isize::MAX as u64 {
            Err(Error::out_of_bounds(format!(
                "{allocation_size} overflows isize::MAX"
            )))?
        }
        device.increment_memory_alloc_count()?;
        let mut handle = None;
        let result = unsafe {
            (device.fun.allocate_memory)(
                device.handle(),
                &MemoryAllocateInfo {
                    stype: Default::default(),
                    next: Default::default(),
                    allocation_size,
                    memory_type_index,
                },
                None,
                &mut handle,
            )
            .context("vkAllocateMemory")
        };
        if result.is_err() {
            device.decrement_memory_alloc_count();
            result?;
        }
        Ok(Self {
            allocation_size,
            memory_type_index,
            inner: Owner::new(MemoryLifetime {
                handle: handle.unwrap(),
                device: device.clone(),
            }),
        })
    }

    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDeviceMemory> {
        self.inner.handle.borrow()
    }
    /// Borrows the inner Vulkan handle.
    pub fn mut_handle(&mut self) -> Mut<VkDeviceMemory> {
        self.inner.handle.borrow_mut()
    }
    /// Returns the associated device.
    pub fn device(&self) -> &Arc<Device> {
        &self.inner.device
    }
    /// Extend the lifetime of the memory until the returned object is dropped.
    pub(crate) fn resource(&self) -> Subobject<MemoryLifetime> {
        Subobject::new(&self.inner)
    }
    pub(crate) fn bounds_check(&self, offset: u64, size: u64) -> Result<()> {
        if self.allocation_size < offset || self.allocation_size - offset < size
        {
            Err(Error::invalid_argument(format!(
                "Size {size} and offset {offset} overflows memory size {}",
                self.allocation_size
            )))?
        }
        Ok(())
    }
    /// Check if the memory meets `requirements` at the given offset.
    pub fn check(
        &self, offset: u64, requirements: MemoryRequirements,
    ) -> Result<()> {
        self.bounds_check(offset, requirements.size)?;
        requirements.check_type(self.memory_type_index)?;
        if offset & (requirements.alignment - 1) != 0 {
            Err(Error::invalid_argument(format!(
                "Offset {:x} not aligned to {} bytes",
                offset, requirements.alignment
            )))?
        }
        Ok(())
    }
}

impl Drop for MemoryLifetime {
    fn drop(&mut self) {
        unsafe {
            (self.device.fun.free_memory)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
        self.device.decrement_memory_alloc_count();
    }
}

/// A [`DeviceMemory`] which has been mapped and can be written to
pub struct MappedMemory {
    memory: DeviceMemory,
    size: usize,
    ptr: NonNull<u8>,
}

/// A structure for copying data out of mapped memory. Implements
/// [`std::io::Read`].
pub struct MemoryRead<'a> {
    ptr: NonNull<u8>,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}
/// A structure for copying data into mapped memory. Implements
/// [`std::io::Write`].
pub struct MemoryWrite<'a> {
    ptr: NonNull<u8>,
    end: *const u8,
    _lt: PhantomData<&'a ()>,
}

#[allow(clippy::len_without_is_empty)]
impl DeviceMemory {
    /// Map the memory so it can be written to. Returns [`ErrorKind::OutOfBounds`] if
    /// `offset` and `size` are out of bounds.
    pub fn map(
        mut self, offset: u64, size: usize,
    ) -> ResultAnd<MappedMemory, Self> {
        if let Err(err) = self.bounds_check(offset, size as u64) {
            Err(err.and(self))?
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
            )
            .context("vkMapMemory")
            {
                Err(err.and(self))?
            }
        }
        Ok(MappedMemory { memory: self, size, ptr: NonNull::new(ptr).unwrap() })
    }
    /// Returns the size of the memory in bytes.
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

    /// Unmaps the memory
    pub fn unmap(mut self) -> DeviceMemory {
        self.unmap_impl();
        let no_drop = std::mem::ManuallyDrop::new(self);
        unsafe { std::ptr::addr_of!(no_drop.memory).read() }
    }

    /// Gets the associated memory object
    pub fn memory(&self) -> &DeviceMemory {
        &self.memory
    }

    /// Read the memory's contents. It may be garbage (although it won't be
    /// uninitialized). If `offset` is out of bounds, the result will be empty.
    #[inline]
    pub fn read_at(&self, offset: usize) -> MemoryRead {
        unsafe {
            let ptr = self.ptr.as_ptr().add(offset.min(self.size));
            MemoryRead {
                ptr: NonNull::new_unchecked(ptr),
                end: self.ptr.as_ptr().add(self.size),
                _lt: PhantomData,
            }
        }
    }

    /// Write to the memory. If `offset` is out of bounds, the result will be
    /// empty.
    #[inline]
    pub fn write_at(&mut self, offset: usize) -> MemoryWrite {
        unsafe {
            let ptr = self.ptr.as_ptr().add(offset.min(self.size));
            MemoryWrite {
                ptr: NonNull::new_unchecked(ptr),
                end: self.ptr.as_ptr().add(self.size),
                _lt: PhantomData,
            }
        }
    }
}

impl<'a> std::io::Read for MemoryRead<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            let count = size.min(buf.len());
            std::ptr::copy_nonoverlapping(
                self.ptr.as_ptr(),
                buf.as_mut_ptr(),
                count,
            );
            self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(count));
            Ok(count)
        }
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            buf.reserve_exact(size);
            std::ptr::copy_nonoverlapping(
                self.ptr.as_ptr(),
                buf.spare_capacity_mut().as_mut_ptr() as *mut u8,
                size,
            );
            buf.set_len(buf.len() + size);
            Ok(size)
        }
    }
}

impl<'a> std::io::Write for MemoryWrite<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            let size = self.end.offset_from(self.ptr.as_ptr()) as usize;
            let count = size.min(buf.len());
            std::ptr::copy_nonoverlapping(
                buf.as_ptr(),
                self.ptr.as_ptr(),
                count,
            );
            self.ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(count));
            Ok(count)
        }
    }
    /// Returns an error of kind [`std::io::ErrorKind::WriteZero`] if not all
    /// the bytes could be written.
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if self.write(buf)? == buf.len() {
            Ok(())
        } else {
            Err(std::io::ErrorKind::WriteZero.into())
        }
    }

    /// Does nothing.
    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// Access to ptr is properly controlled with borrows
unsafe impl Send for MappedMemory {}
unsafe impl Sync for MappedMemory {}
impl std::panic::UnwindSafe for MappedMemory {}
impl std::panic::RefUnwindSafe for MappedMemory {}
unsafe impl<'a> Send for MemoryRead<'a> {}
unsafe impl<'a> Sync for MemoryRead<'a> {}
impl<'a> std::panic::UnwindSafe for MemoryRead<'a> {}
impl<'a> std::panic::RefUnwindSafe for MemoryRead<'a> {}
unsafe impl<'a> Send for MemoryWrite<'a> {}
unsafe impl<'a> Sync for MemoryWrite<'a> {}
impl<'a> std::panic::UnwindSafe for MemoryWrite<'a> {}
impl<'a> std::panic::RefUnwindSafe for MemoryWrite<'a> {}
