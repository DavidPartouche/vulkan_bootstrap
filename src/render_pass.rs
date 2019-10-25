use std::rc::Rc;

use ash::vk;

use crate::device::VulkanDevice;
use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

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
    context: &'a VulkanContext,
}

impl<'a> RenderPassBuilder<'a> {
    pub fn new(context: &'a VulkanContext) -> Self {
        RenderPassBuilder { context }
    }

    pub fn build(self) -> Result<RenderPass, VulkanError> {
        let color_attachment = vk::AttachmentDescription::builder()
            .format(self.context.get_swapchain().get_format().format)
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
            .format(self.context.get_depth_resources().get_format())
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

        let subpasses = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&[color_attachment_ref])
                .depth_stencil_attachment(&depth_attachment_ref)
                .build(),
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&[color_attachment_ref])
                .depth_stencil_attachment(&depth_attachment_ref)
                .build(),
        ];

        let dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::MEMORY_READ)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
            vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(1)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE)
                .src_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ
                        | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                .dependency_flags(vk::DependencyFlags::BY_REGION)
                .build(),
        ];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment, depth_attachment])
            .subpasses(&subpasses)
            .dependencies(&dependencies)
            .build();

        let render_pass = self
            .context
            .get_device()
            .create_render_pass(&render_pass_info)?;

        Ok(RenderPass {
            device: Rc::clone(self.context.get_device()),
            render_pass,
        })
    }
}
