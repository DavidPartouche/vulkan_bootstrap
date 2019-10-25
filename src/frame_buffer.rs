use std::rc::Rc;

use ash::vk;

use crate::depth_resources::DepthResources;
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::render_pass::RenderPass;
use crate::swapchain::Swapchain;

pub struct FrameBuffers {
    device: Rc<VulkanDevice>,
    frame_buffers: Vec<vk::Framebuffer>,
}

impl Drop for FrameBuffers {
    fn drop(&mut self) {
        for frame_buffer in self.frame_buffers.iter() {
            self.device.destroy_frame_buffer(*frame_buffer);
        }
    }
}

impl FrameBuffers {
    pub fn get(&self, index: usize) -> vk::Framebuffer {
        self.frame_buffers[index]
    }
}

pub struct FrameBuffersBuilder<'a> {
    device: Rc<VulkanDevice>,
    render_pass: &'a RenderPass,
    swapchain: &'a Swapchain,
    depth_resources: &'a DepthResources,
    width: u32,
    height: u32,
    frames_count: u32,
}

impl<'a> FrameBuffersBuilder<'a> {
    pub fn new(
        device: Rc<VulkanDevice>,
        render_pass: &'a RenderPass,
        swapchain: &'a Swapchain,
        depth_resources: &'a DepthResources,
    ) -> Self {
        FrameBuffersBuilder {
            device,
            render_pass,
            swapchain,
            depth_resources,
            width: 0,
            height: 0,
            frames_count: 1,
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

    pub fn with_frames_count(mut self, frames_count: u32) -> Self {
        self.frames_count = frames_count;
        self
    }

    pub fn build(self) -> Result<FrameBuffers, VulkanError> {
        let mut frame_buffers = vec![];

        for i in 0..self.frames_count {
            let image_view = self.swapchain.get_image_view(i as usize);
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(self.render_pass.get())
                .attachments(&[image_view, self.depth_resources.get_image_view()])
                .width(self.width)
                .height(self.height)
                .layers(1)
                .build();

            frame_buffers.push(self.device.create_frame_buffer(&framebuffer_info)?);
        }

        Ok(FrameBuffers {
            device: self.device,
            frame_buffers,
        })
    }
}
