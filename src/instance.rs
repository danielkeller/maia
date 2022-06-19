use crate::error::Result;
use crate::load;
use crate::load::InstanceFn;
use crate::types::*;

pub struct Instance {
    handle: Handle<VkInstance>,
    pub(crate) fun: InstanceFn,
}

impl std::fmt::Debug for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.handle.fmt(f)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { (self.fun.destroy_instance)(self.handle.borrow_mut(), None) }
    }
}

impl Instance {
    /// Creates a new instance
    #[doc = crate::man_link!(vkCreateInstance)]
    pub fn new<'a>(info: &'a InstanceCreateInfo<'a>) -> Result<Arc<Self>> {
        let mut handle = None;
        unsafe { (load::vk_create_instance())(info, None, &mut handle)? };
        let handle = handle.unwrap();
        let fun = InstanceFn::new(handle.borrow());
        Ok(Arc::new(Instance { handle, fun }))
    }
    /// Borrows the inner Vulkan handle.
    pub fn handle(&self) -> Ref<VkInstance> {
        self.handle.borrow()
    }
}
