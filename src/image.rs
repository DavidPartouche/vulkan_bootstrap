use ash::vk;

use crate::errors::VulkanError;
use crate::vulkan_context::VulkanContext;

pub fn create_image(
    context: &VulkanContext,
    width: u32,
    height: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory), VulkanError> {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(
            vk::Extent3D::builder()
                .width(width)
                .height(height)
                .depth(1)
                .build(),
        )
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .build();

    let image = context.get_device().create_image(&image_info)?;
    let mem_requirements = context.get_device().get_image_memory_requirements(image);

    let memory_type_index = context
        .get_instance()
        .find_memory_type(
            context.get_physical_device().get(),
            mem_requirements.memory_type_bits,
            properties,
        )
        .ok_or_else(|| {
            VulkanError::ImageCreationError(String::from("Cannot find a memory type"))
        })?;

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(mem_requirements.size)
        .memory_type_index(memory_type_index)
        .build();
    let image_memory = context.get_device().allocate_memory(&alloc_info)?;

    context
        .get_device()
        .bind_image_memory(image, image_memory)?;

    Ok((image, image_memory))
}

pub fn create_image_view(
    context: &VulkanContext,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
) -> Result<vk::ImageView, VulkanError> {
    let view_info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_flags)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .build();

    context.get_device().create_image_view(&view_info)
}

pub fn transition_image_layout(
    context: &VulkanContext,
    image: vk::Image,
    format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) -> Result<(), VulkanError> {
    let command_buffer = context.begin_single_time_commands()?;

    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        if format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        } else {
            vk::ImageAspectFlags::DEPTH
        }
    } else {
        vk::ImageAspectFlags::COLOR
    };

    let (src_access_mask, dst_access_mask, src_stage, dst_stage) = if old_layout
        == vk::ImageLayout::UNDEFINED
        && (new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            || new_layout == vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
    {
        (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        )
    } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        )
    } else if old_layout == vk::ImageLayout::UNDEFINED
        && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
    {
        (
            vk::AccessFlags::empty(),
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
    } else {
        return Err(VulkanError::ImageCreationError(String::from(
            "unsupported layout transition",
        )));
    };

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        )
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .build();

    context.get_device().cmd_pipeline_barrier(
        command_buffer,
        src_stage,
        dst_stage,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
    );

    context.end_single_time_commands(command_buffer)
}
