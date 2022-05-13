use crate::device::Device;
use crate::enums::*;
use crate::error::Error;
use crate::error::Result;
use crate::image::Image;
use crate::queue::Queue;
use crate::semaphore::Semaphore;
use crate::types::*;

use super::khr_surface::SurfaceLifetime;
use super::load::SwapchainDeviceFn;
use super::load::SwapchainKHRFn;
use super::SurfaceKHR;

pub struct KHRSwapchain {
    fun: SwapchainDeviceFn,
    device: Arc<Device>,
}

impl Device {
    pub fn khr_swapchain(self: &Arc<Self>) -> KHRSwapchain {
        KHRSwapchain {
            fun: SwapchainDeviceFn::new(self),
            device: self.clone(),
        }
    }
}

pub enum CreateSwapchainFrom {
    OldSwapchain(SwapchainKHR),
    Surface(SurfaceKHR),
}

pub struct SwapchainCreateInfoKHR<'a> {
    pub flags: SwapchainCreateFlagsKHR,
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpaceKHR,
    pub image_extent: Extent2D,
    pub image_array_layers: u32,
    pub image_usage: ImageUsageFlags,
    pub image_sharing_mode: SharingMode,
    pub queue_family_indices: &'a [u32],
    pub pre_transform: SurfaceTransformKHR,
    pub composite_alpha: CompositeAlphaKHR,
    pub present_mode: PresentModeKHR,
    pub clipped: Bool,
}

impl<'a> Default for SwapchainCreateInfoKHR<'a> {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            min_image_count: Default::default(),
            image_format: Default::default(),
            image_color_space: Default::default(),
            image_extent: Default::default(),
            image_array_layers: 1,
            image_usage: Default::default(),
            image_sharing_mode: Default::default(),
            queue_family_indices: Default::default(),
            pre_transform: Default::default(),
            composite_alpha: Default::default(),
            present_mode: Default::default(),
            clipped: Default::default(),
        }
    }
}

impl KHRSwapchain {
    pub fn create(
        &self,
        create_from: CreateSwapchainFrom,
        info: SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR> {
        let (mut surface, mut old_swapchain) = match create_from {
            CreateSwapchainFrom::OldSwapchain(old) => {
                (old.surface, Some(old.handle))
            }
            CreateSwapchainFrom::Surface(surf) => (surf, None),
        };
        let mut handle = None;
        unsafe {
            (self.fun.create_swapchain_khr)(
                self.device.borrow(),
                &VkSwapchainCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: info.flags,
                    surface: surface.borrow_mut(),
                    min_image_count: info.min_image_count,
                    image_format: info.image_format,
                    image_color_space: info.image_color_space,
                    image_extent: info.image_extent,
                    image_array_layers: info.image_array_layers,
                    image_usage: info.image_usage,
                    image_sharing_mode: info.image_sharing_mode,
                    queue_family_indices: info.queue_family_indices.into(),
                    pre_transform: info.pre_transform,
                    composite_alpha: info.composite_alpha,
                    present_mode: info.present_mode,
                    clipped: info.clipped,
                    old_swapchain: old_swapchain
                        .as_mut()
                        .map(|h| h.borrow_mut()),
                },
                None,
                &mut handle,
            )?;
        }
        let fun = SwapchainKHRFn::new(&self.device);
        let handle = handle.unwrap();

        let mut n_images = 0;
        let mut images = vec![];
        unsafe {
            (fun.get_swapchain_images_khr)(
                self.device.borrow(),
                handle.borrow(),
                &mut n_images,
                None,
            )?;
            images.reserve(n_images as usize);
            (fun.get_swapchain_images_khr)(
                self.device.borrow(),
                handle.borrow(),
                &mut n_images,
                images.spare_capacity_mut().first_mut(),
            )?;
            images.set_len(n_images as usize);
        }

        let res = Arc::new(SwapchainImages {
            // Safety: Only used after the SwapchainKHR is destroyed.
            _handle: unsafe { handle.clone() },
            fun,
            device: self.device.clone(),
            _surface: surface.res.clone(),
        });
        let images = images.into_boxed_slice();

        Ok(SwapchainKHR { handle, res, surface, images })
    }
}

// Conceptually this owns the images, but it's also used to delay destruction
// of the swapchain until it's no longer used by the images.
struct SwapchainImages {
    /// Safety: Only use in Drop::drop
    _handle: Handle<VkSwapchainKHR>,
    fun: SwapchainKHRFn,
    device: Arc<Device>,
    // Needs to be destroyed after the swapchain
    _surface: Arc<SurfaceLifetime>,
}

impl Drop for SwapchainImages {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_swapchain_khr)(
                self.device.borrow(),
                self._handle.borrow_mut(),
                None,
            )
        }
    }
}

impl std::fmt::Debug for SwapchainImages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapchainImages")
            .field("_handle", &self._handle)
            .finish()
    }
}

#[derive(Debug)]
pub struct SwapchainKHR {
    handle: Handle<VkSwapchainKHR>,
    images: Box<[Handle<VkImage>]>,
    res: Arc<SwapchainImages>,
    surface: SurfaceKHR,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageOptimality {
    Optimal,
    Suboptimal,
}

impl SwapchainKHR {
    pub fn borrow_mut(&mut self) -> Mut<VkSwapchainKHR> {
        self.handle.borrow_mut()
    }
    pub fn surface(&self) -> &SurfaceKHR {
        &self.surface
    }

    pub fn acquire_next_image(
        &mut self,
        signal: &mut Semaphore,
        timeout: u64,
    ) -> Result<(Image, ImageOptimality)> {
        let mut index = 0;
        let res = unsafe {
            (self.res.fun.acquire_next_image_khr)(
                self.res.device.borrow(),
                self.handle.borrow_mut(),
                timeout,
                Some(signal.borrow_mut()),
                None,
                &mut index,
            )
        };
        // Safety: You can't acquire the same image twice.
        let handle = unsafe { self.images[index as usize].clone() };
        let image = Image::new(handle, self.res.clone());
        match res {
            Ok(()) => Ok((image, ImageOptimality::Optimal)),
            Err(e) => match e.into() {
                Error::SuboptimalHKR => {
                    Ok((image, ImageOptimality::Suboptimal))
                }
                other => Err(other),
            },
        }
    }

    pub fn present(
        &mut self,
        queue: &mut Queue,
        image: Image,
        wait: &mut Semaphore,
    ) -> Result<ImageOptimality> {
        let index = self
            .images
            .iter()
            .position(|h| h.borrow() == image.borrow())
            .ok_or(Error::InvalidArgument)?
            .try_into()
            .unwrap();

        let res = unsafe {
            (self.res.fun.queue_present_khr)(
                queue.borrow_mut(),
                &PresentInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    wait: (&[wait.borrow_mut()]).into(),
                    swapchain_count: 1,
                    swapchains: &self.borrow_mut(),
                    indices: &index,
                    result: None,
                },
            )
        };
        match res {
            Ok(()) => Ok(ImageOptimality::Optimal),
            Err(e) => match e.into() {
                Error::SuboptimalHKR => Ok(ImageOptimality::Suboptimal),
                other => Err(other),
            },
        }
    }
}
