use crate::error::Result;
use crate::instance::Instance;
use crate::load::DeviceFn;
use crate::physical_device::PhysicalDevice;
use crate::types::*;

/// A logical device.
pub struct Device {
    handle: Handle<VkDevice>,
    pub(crate) fun: DeviceFn,
    physical_device: PhysicalDevice,
    queues: Vec<u32>,
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool {
        self.handle == other.handle
    }
}
impl Eq for Device {}
impl std::hash::Hash for Device {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.handle.hash(state)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            (self.fun.device_wait_idle)(self.handle.borrow_mut()).unwrap();
            (self.fun.destroy_device)(self.handle.borrow_mut(), None);
        }
    }
}

impl PhysicalDevice {
    /// Create a logical device for this physical device.
    #[doc = crate::man_link!(vkCreateDevice)]
    pub fn create_device(
        &self,
        info: &DeviceCreateInfo<'_>,
    ) -> Result<Arc<Device>> {
        let props = self.queue_family_properties();
        let mut queues = vec![0; props.len()];
        for q in info.queue_create_infos.as_slice() {
            let i = q.queue_family_index as usize;
            assert!(i < props.len(), "Queue family index out of bounds");
            assert!(
                q.queue_priorities.len() <= props[i].queue_count,
                "Too many queues requested"
            );
            queues[i] = q.queue_priorities.len();
        }

        let mut handle = None;
        unsafe {
            (self.instance().fun.create_device)(
                self.handle(),
                info,
                None,
                &mut handle,
            )?;
        }
        let handle = handle.unwrap();
        let fun = DeviceFn::new(self.instance(), handle.borrow());
        Ok(Arc::new(Device {
            handle,
            fun,
            physical_device: self.clone(),
            queues,
        }))
    }
}

impl Device {
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkDevice> {
        self.handle.borrow()
    }
    /// Returns the associated phyical device.
    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
    /// Returns the associated instance.
    pub fn instance(&self) -> &Instance {
        self.physical_device.instance()
    }
    /// Returns true if a queue with this family index and index exists.
    pub fn has_queue(&self, queue_family_index: u32, queue_index: u32) -> bool {
        let i = queue_family_index as usize;
        i < self.queues.len() && self.queues[i] >= queue_index
    }
}
