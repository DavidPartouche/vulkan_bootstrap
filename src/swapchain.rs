use std::rc::Rc;

use ash::extensions::khr;
use ash::vk;

use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;
use std::mem;

pub struct Swapchain {
    device: Rc<VulkanDevice>,
    swapchain_loader: Option<khr::Swapchain>,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::SurfaceFormatKHR,
    swapchain_extent: vk::Extent2D,
    swapchain_images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for image_view in self.image_views.iter() {
                self.device.destroy_image_view(*image_view);
            }
            self.swapchain_loader
                .as_ref()
                .unwrap()
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Swapchain {
    pub fn get(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn get_image(&self, index: usize) -> vk::Image {
        self.swapchain_images[index]
    }

    pub fn get_image_view(&self, index: usize) -> vk::ImageView {
        self.image_views[index]
    }

    pub fn get_format(&self) -> vk::SurfaceFormatKHR {
        self.swapchain_format
    }

    pub fn get_extent(&self) -> vk::Extent2D {
        self.swapchain_extent
    }

    pub fn acquire_next_image(&self, semaphore: vk::Semaphore) -> Result<usize, VulkanError> {
        let (index, _) = unsafe {
            self.swapchain_loader.as_ref().unwrap().acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                semaphore,
                vk::Fence::null(),
            )
        }
        .map_err(|err| VulkanError::SwapchainError(err.to_string()))?;
        Ok(index as usize)
    }

    pub fn queue_present(
        &self,
        semaphore: vk::Semaphore,
        image_index: u32,
    ) -> Result<(), VulkanError> {
        let info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[semaphore])
            .swapchains(&[self.swapchain])
            .image_indices(&[image_index])
            .build();
        unsafe {
            self.swapchain_loader
                .as_ref()
                .unwrap()
                .queue_present(self.device.get_queue(), &info)
        }
        .map_err(|err| VulkanError::SwapchainError(err.to_string()))?;

        Ok(())
    }
}

pub struct SwapchainBuilder<'a> {
    context: &'a VulkanContext,
    old_swapchain: Option<Swapchain>,
    frames_count: u32,
    width: u32,
    height: u32,
}

impl<'a> SwapchainBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        SwapchainBuilder {
            context,
            old_swapchain: None,
            frames_count: 1,
            width: 0,
            height: 0,
        }
    }

    pub fn with_old_swapchain(mut self, old_swapchain: Option<Swapchain>) -> Self {
        self.old_swapchain = old_swapchain;
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn with_frames_count(mut self, frames_count: u32) -> Self {
        self.frames_count = frames_count;
        self
    }

    pub fn build(mut self) -> Result<Swapchain, VulkanError> {
        let swapchain_format = self.choose_surface_format()?;
        let present_mode = self.choose_present_mode()?;
        let swapchain_extent = self.choose_surface_extent()?;

        let old_swapchain = if self.old_swapchain.is_some() {
            self.old_swapchain.as_ref().unwrap().get()
        } else {
            vk::SwapchainKHR::null()
        };

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(self.context.get_surface().get())
            .min_image_count(self.frames_count)
            .image_format(swapchain_format.format)
            .image_color_space(swapchain_format.color_space)
            .image_extent(swapchain_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::STORAGE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .old_swapchain(old_swapchain)
            .build();

        let swapchain_loader = if self.old_swapchain.is_some() {
            self.old_swapchain.as_mut().unwrap().swapchain_loader.take()
        } else {
            Some(self.context.get_device().new_swapchain())
        };

        let swapchain = unsafe {
            swapchain_loader
                .as_ref()
                .unwrap()
                .create_swapchain(&info, None)
        }
        .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        if let Some(old_swapchain) = self.old_swapchain.take() {
            mem::drop(old_swapchain);
        }

        let swapchain_images = unsafe {
            swapchain_loader
                .as_ref()
                .unwrap()
                .get_swapchain_images(swapchain)
        }
        .map_err(|err| VulkanError::SwapchainCreationError(err.to_string()))?;

        let image_views = swapchain_images
            .iter()
            .map(|image| {
                let view_info = vk::ImageViewCreateInfo::builder()
                    .image(*image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain_format.format)
                    .components(
                        vk::ComponentMapping::builder()
                            .r(vk::ComponentSwizzle::R)
                            .g(vk::ComponentSwizzle::G)
                            .b(vk::ComponentSwizzle::B)
                            .a(vk::ComponentSwizzle::A)
                            .build(),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    )
                    .build();

                self.context
                    .get_device()
                    .create_image_view(&view_info)
                    .unwrap()
            })
            .collect();

        Ok(Swapchain {
            device: Rc::clone(self.context.get_device()),
            swapchain_loader,
            swapchain,
            swapchain_format,
            swapchain_extent,
            swapchain_images,
            image_views,
        })
    }

    fn choose_surface_format(&self) -> Result<vk::SurfaceFormatKHR, VulkanError> {
        let formats = self
            .context
            .get_surface()
            .get_physical_device_surface_formats(self.context.get_physical_device().get())?;

        Ok(
            if formats.len() == 1 && formats[0].format == vk::Format::UNDEFINED {
                vk::SurfaceFormatKHR::builder()
                    .format(vk::Format::B8G8R8A8_UNORM)
                    .color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
                    .build()
            } else {
                formats
                    .iter()
                    .find_map(|format| {
                        if format.format == vk::Format::B8G8R8A8_UNORM
                            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                        {
                            Some(*format)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(formats[0])
            },
        )
    }

    fn choose_present_mode(&self) -> Result<vk::PresentModeKHR, VulkanError> {
        let present_modes = self
            .context
            .get_surface()
            .get_physical_device_surface_present_modes(self.context.get_physical_device().get())?;

        Ok(present_modes
            .into_iter()
            .find(|mode| *mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO))
    }

    fn choose_surface_extent(&self) -> Result<vk::Extent2D, VulkanError> {
        let caps = self
            .context
            .get_surface()
            .get_physical_device_surface_capabilities(self.context.get_physical_device().get())?;

        Ok(if caps.current_extent.width == std::u32::MAX {
            vk::Extent2D {
                width: self.width,
                height: self.height,
            }
        } else {
            caps.current_extent
        })
    }
}
