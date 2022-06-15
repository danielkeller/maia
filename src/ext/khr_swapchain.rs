use crate::device::Device;
use crate::enums::*;
use crate::error::{Error, Result};
use crate::ffi::ArrayMut;
use crate::image::Image;
use crate::queue::Queue;
use crate::semaphore::{Semaphore, SemaphoreSignaller};
use crate::subobject::{Owner, Subobject};
use crate::types::*;

use super::khr_surface::SurfaceLifetime;
use super::load::{SwapchainDeviceFn, SwapchainKHRFn};
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
    /// If create_from is OldSwapchain(), images in that swapchain that aren't
    /// acquired by the application are deleted. If any references remain to
    /// those images, returns SynchronizationError.
    pub fn create(
        &self,
        create_from: CreateSwapchainFrom,
        info: SwapchainCreateInfoKHR,
    ) -> Result<SwapchainKHR> {
        let (mut surface, mut old_swapchain) = match create_from {
            CreateSwapchainFrom::OldSwapchain(mut old) => {
                for (img, acquired) in &mut old.images {
                    if !*acquired && Arc::get_mut(img).is_none() {
                        return Err(Error::SynchronizationError);
                    }
                }
                (old.surface, Some(old.res))
            }
            CreateSwapchainFrom::Surface(surf) => (surf, None),
        };
        let mut handle = None;
        unsafe {
            (self.fun.create_swapchain_khr)(
                self.device.handle(),
                &VkSwapchainCreateInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    flags: info.flags,
                    surface: surface.handle_mut(),
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
                        .map(|h| h.handle.borrow_mut()),
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
                self.device.handle(),
                handle.borrow(),
                &mut n_images,
                None,
            )?;
            images.reserve(n_images as usize);
            (fun.get_swapchain_images_khr)(
                self.device.handle(),
                handle.borrow(),
                &mut n_images,
                ArrayMut::from_slice(images.spare_capacity_mut()),
            )?;
            images.set_len(n_images as usize);
        }

        let res = Owner::new(SwapchainImages {
            handle,
            fun,
            device: self.device.clone(),
            _surface: surface.resource(),
        });
        let images = images
            .into_iter()
            .map(|h| {
                (
                    Arc::new(Image::new(
                        h,
                        self.device.clone(),
                        Subobject::new(&res),
                        info.image_format,
                        info.image_extent.into(),
                        info.image_array_layers,
                    )),
                    false,
                )
            })
            .collect();

        Ok(SwapchainKHR { res, surface, images })
    }
}

// Conceptually this owns the images, but it's also used to delay destruction
// of the swapchain until it's no longer used by the images.
pub(crate) struct SwapchainImages {
    handle: Handle<VkSwapchainKHR>,
    fun: SwapchainKHRFn,
    device: Arc<Device>,
    _surface: Subobject<SurfaceLifetime>,
}

impl Drop for SwapchainImages {
    fn drop(&mut self) {
        unsafe {
            (self.fun.destroy_swapchain_khr)(
                self.device.handle(),
                self.handle.borrow_mut(),
                None,
            )
        }
    }
}

impl std::fmt::Debug for SwapchainImages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwapchainImages").field("handle", &self.handle).finish()
    }
}

#[derive(Debug)]
pub struct SwapchainKHR {
    images: Vec<(Arc<Image>, bool)>,
    res: Owner<SwapchainImages>,
    surface: SurfaceKHR,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageOptimality {
    Optimal,
    Suboptimal,
}

impl SwapchainKHR {
    pub fn handle_mut(&mut self) -> Mut<VkSwapchainKHR> {
        self.res.handle.borrow_mut()
    }
    pub fn surface(&self) -> &SurfaceKHR {
        &self.surface
    }

    pub fn acquire_next_image(
        &mut self,
        signal: &mut Semaphore,
        timeout: u64,
    ) -> Result<(Arc<Image>, ImageOptimality)> {
        let mut index = 0;
        let res = &mut *self.res;
        let res = unsafe {
            (res.fun.acquire_next_image_khr)(
                res.device.handle(),
                res.handle.borrow_mut(),
                timeout,
                Some(signal.handle_mut()),
                None,
                &mut index,
            )
        };
        let is_optimal = match res {
            Ok(()) => ImageOptimality::Optimal,
            Err(e) => match e.into() {
                Error::SuboptimalHKR => ImageOptimality::Suboptimal,
                other => return Err(other),
            },
        };
        let (image, acquired) = &mut self.images[index as usize];
        *acquired = true;
        let image = image.clone();
        signal.signaller = Some(SemaphoreSignaller::Swapchain(image.clone()));
        Ok((image, is_optimal))
    }

    pub fn present(
        &mut self,
        queue: &mut Queue,
        image: &Image,
        wait: &mut Semaphore,
    ) -> Result<ImageOptimality> {
        let index = self
            .images
            .iter()
            .position(|h| h.0.handle() == image.handle())
            .ok_or(Error::InvalidArgument)?;
        if wait.signaller.is_none() {
            return Err(Error::InvalidArgument);
        }
        let acquired = &mut self.images[index].1;

        let res = unsafe {
            (self.res.fun.queue_present_khr)(
                queue.handle_mut(),
                &PresentInfoKHR {
                    stype: Default::default(),
                    next: Default::default(),
                    wait: (&[wait.handle_mut()]).into(),
                    swapchains: (&[self.res.handle.borrow_mut()]).into(),
                    indices: (&[index as u32]).into(),
                    results: None,
                },
            )
        };
        let is_optimal = match res {
            Ok(()) => ImageOptimality::Optimal,
            Err(e) => match e.into() {
                Error::SuboptimalHKR => ImageOptimality::Suboptimal,
                other => return Err(other),
            },
        };
        *acquired = false;
        queue.add_resource(wait.take_signaller());
        queue.add_resource(wait.inner.clone());
        Ok(is_optimal)
    }
}
