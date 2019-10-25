use std::rc::Rc;

use ash::vk;

use crate::depth_resources::DepthResources;
use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::swapchain::Swapchain;

pub struct RenderPass {
    device: Rc<VulkanDevice>,
    render_pass: vk::RenderPass,
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        self.device.destroy_render_pass(self.render_pass);
    }
}

impl RenderPass {
    pub fn get(&self) -> vk::RenderPass {
        self.render_pass
    }
}

pub struct RenderPassBuilder<'a> {
    device: Rc<VulkanDevice>,
    swapchain: &'a Swapchain,
    depth_resources: &'a DepthResources,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new(
        device: Rc<VulkanDevice>,
        swapchain: &'a Swapchain,
        depth_resources: &'a DepthResources,
    ) -> Self {
        RenderPassBuilder {
            device,
            swapchain,
            depth_resources,
        }
    }

    pub fn build(self) -> Result<RenderPass, VulkanError> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(self.swapchain.get_format().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment = vk::AttachmentDescription::builder()
            .format(self.depth_resources.get_format())
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            .depth_stencil_attachment(&depth_attachment_ref)
            .build();

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment, depth_attachment])
            .subpasses(&[subpass])
            .build();

        let render_pass = self.device.create_render_pass(&render_pass_info)?;

        Ok(RenderPass {
            device: self.device,
            render_pass,
        })
    }
}
