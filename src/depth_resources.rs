use std::rc::Rc;

use ash::vk;

use crate::command_buffers::CommandBuffers;
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::image;
use crate::instance::VulkanInstance;
use crate::physical_device::PhysicalDevice;

pub struct DepthResources {
    device: Rc<VulkanDevice>,
    depth_format: vk::Format,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
}

impl Drop for DepthResources {
    fn drop(&mut self) {
        self.device.destroy_image_view(self.depth_image_view);
        self.device.destroy_image(self.depth_image);
        self.device.free_memory(self.depth_image_memory);
    }
}

impl DepthResources {
    pub fn get_format(&self) -> vk::Format {
        self.depth_format
    }

    pub fn get_image_view(&self) -> vk::ImageView {
        self.depth_image_view
    }
}

pub struct DepthResourcesBuilder<'a> {
    instance: &'a VulkanInstance,
    physical_device: &'a PhysicalDevice,
    device: Rc<VulkanDevice>,
    command_buffers: &'a CommandBuffers,
    width: u32,
    height: u32,
}

impl<'a> DepthResourcesBuilder<'a> {
    pub fn new(
        instance: &'a VulkanInstance,
        physical_device: &'a PhysicalDevice,
        device: Rc<VulkanDevice>,
        command_buffers: &'a CommandBuffers,
    ) -> Self {
        DepthResourcesBuilder {
            instance,
            physical_device,
            device,
            command_buffers,
            width: 0,
            height: 0,
        }
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn build(self) -> Result<DepthResources, VulkanError> {
        let depth_format = self.choose_supported_format(
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        );

        let (depth_image, depth_image_memory) = image::create_image(
            self.instance,
            &self.device,
            &self.physical_device,
            self.width,
            self.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view = image::create_image_view(
            &self.device,
            depth_image,
            depth_format,
            vk::ImageAspectFlags::DEPTH,
        )?;

        image::transition_image_layout(
            &self.device,
            self.command_buffers,
            depth_image,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok(DepthResources {
            device: self.device,
            depth_format,
            depth_image,
            depth_image_memory,
            depth_image_view,
        })
    }

    fn choose_supported_format(
        &self,
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        let candidates = vec![
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        candidates
            .into_iter()
            .find(|format| {
                let props = self
                    .instance
                    .get_physical_device_format_properties(self.physical_device.get(), *format);

                (tiling == vk::ImageTiling::LINEAR
                    && props.linear_tiling_features.contains(features))
                    || (tiling == vk::ImageTiling::OPTIMAL
                        && props.optimal_tiling_features.contains(features))
            })
            .unwrap_or(vk::Format::UNDEFINED)
    }
}
