use std::os::raw::{c_char, c_void};
use std::rc::Rc;

use ash::extensions::khr;
use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::PhysicalDeviceDescriptorIndexingFeaturesEXT;

use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::features::Features;
use crate::instance::VulkanInstance;
use crate::physical_device::PhysicalDevice;

const FENCE_TIMEOUT: u64 = 100;

pub struct VulkanDevice {
    instance: Rc<VulkanInstance>,
    device: ash::Device,
    queue: vk::Queue,
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

impl VulkanDevice {
    pub fn get(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn queue_wait_idle(&self) -> Result<(), VulkanError> {
        unsafe { self.device.queue_wait_idle(self.queue) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))?;

        Ok(())
    }

    pub fn queue_submit(
        &self,
        submit_info: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.queue_submit(self.queue, submit_info, fence) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))?;

        Ok(())
    }

    pub fn create_command_pool(
        &self,
        pool_info: &vk::CommandPoolCreateInfo,
    ) -> Result<vk::CommandPool, VulkanError> {
        unsafe { self.device.create_command_pool(pool_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        unsafe {
            self.device.destroy_command_pool(command_pool, None);
        }
    }

    pub fn allocate_command_buffers(
        &self,
        alloc_info: &vk::CommandBufferAllocateInfo,
    ) -> Result<Vec<vk::CommandBuffer>, VulkanError> {
        unsafe { self.device.allocate_command_buffers(&alloc_info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn free_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffers: &[vk::CommandBuffer],
    ) {
        unsafe {
            self.device
                .free_command_buffers(command_pool, command_buffers);
        }
    }

    pub fn create_fence(&self, fence_info: &vk::FenceCreateInfo) -> Result<vk::Fence, VulkanError> {
        unsafe { self.device.create_fence(&fence_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_fence(&self, fence: vk::Fence) {
        unsafe {
            self.device.destroy_fence(fence, None);
        }
    }

    pub fn create_semaphore(
        &self,
        semaphore_info: &vk::SemaphoreCreateInfo,
    ) -> Result<vk::Semaphore, VulkanError> {
        unsafe { self.device.create_semaphore(&semaphore_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.device.destroy_semaphore(semaphore, None);
        }
    }

    pub fn create_descriptor_pool(
        &self,
        pool_info: &vk::DescriptorPoolCreateInfo,
    ) -> Result<vk::DescriptorPool, VulkanError> {
        unsafe { self.device.create_descriptor_pool(&pool_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_descriptor_pool(&self, descriptor_pool: vk::DescriptorPool) {
        unsafe {
            self.device.destroy_descriptor_pool(descriptor_pool, None);
        }
    }

    pub fn new_swapchain(&self) -> khr::Swapchain {
        khr::Swapchain::new(self.instance.get(), &self.device)
    }

    pub fn create_render_pass(
        &self,
        render_pass_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, VulkanError> {
        unsafe { self.device.create_render_pass(&render_pass_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe {
            self.device.destroy_render_pass(render_pass, None);
        }
    }

    pub fn create_image_view(
        &self,
        view_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, VulkanError> {
        unsafe { self.device.create_image_view(view_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.device.destroy_image_view(image_view, None);
        }
    }

    pub fn create_image(&self, image_info: &vk::ImageCreateInfo) -> Result<vk::Image, VulkanError> {
        unsafe { self.device.create_image(&image_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_image(&self, image: vk::Image) {
        unsafe {
            self.device.destroy_image(image, None);
        }
    }

    pub fn get_image_memory_requirements(&self, image: vk::Image) -> vk::MemoryRequirements {
        unsafe { self.device.get_image_memory_requirements(image) }
    }

    pub fn allocate_memory(
        &self,
        alloc_info: &vk::MemoryAllocateInfo,
    ) -> Result<vk::DeviceMemory, VulkanError> {
        unsafe { self.device.allocate_memory(&alloc_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn free_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.free_memory(memory, None);
        }
    }

    pub fn bind_image_memory(
        &self,
        image: vk::Image,
        memory: vk::DeviceMemory,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.bind_image_memory(image, memory, 0) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn create_frame_buffer(
        &self,
        info: &vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, VulkanError> {
        unsafe { self.device.create_framebuffer(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_frame_buffer(&self, frame_buffer: vk::Framebuffer) {
        unsafe { self.device.destroy_framebuffer(frame_buffer, None) }
    }

    pub fn create_descriptor_set_layout(
        &self,
        layout_info: &vk::DescriptorSetLayoutCreateInfo,
    ) -> Result<vk::DescriptorSetLayout, VulkanError> {
        unsafe { self.device.create_descriptor_set_layout(&layout_info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_descriptor_set_layout(&self, descriptor_set_layout: vk::DescriptorSetLayout) {
        unsafe {
            self.device
                .destroy_descriptor_set_layout(descriptor_set_layout, None);
        }
    }

    pub fn create_pipeline_layout(
        &self,
        info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<vk::PipelineLayout, VulkanError> {
        unsafe { self.device.create_pipeline_layout(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_pipeline_layout(&self, pipeline_layout: vk::PipelineLayout) {
        unsafe {
            self.device.destroy_pipeline_layout(pipeline_layout, None);
        }
    }

    pub fn create_graphics_pipelines(
        &self,
        infos: &[vk::GraphicsPipelineCreateInfo],
    ) -> Result<Vec<vk::Pipeline>, VulkanError> {
        unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), infos, None)
        }
        .map_err(|(_, err)| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.device.destroy_pipeline(pipeline, None);
        }
    }

    pub fn create_shader_module(
        &self,
        info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, VulkanError> {
        unsafe { self.device.create_shader_module(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        unsafe { self.device.destroy_shader_module(shader_module, None) }
    }

    pub fn create_buffer(&self, info: &vk::BufferCreateInfo) -> Result<vk::Buffer, VulkanError> {
        unsafe { self.device.create_buffer(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer) {
        unsafe {
            self.device.destroy_buffer(buffer, None);
        }
    }

    pub fn get_buffer_memory_requirements(&self, buffer: vk::Buffer) -> vk::MemoryRequirements {
        unsafe { self.device.get_buffer_memory_requirements(buffer) }
    }

    pub fn bind_buffer_memory(
        &self,
        buffer: vk::Buffer,
        memory: vk::DeviceMemory,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.bind_buffer_memory(buffer, memory, 0) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn allocate_descriptor_sets(
        &self,
        info: &vk::DescriptorSetAllocateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, VulkanError> {
        unsafe { self.device.allocate_descriptor_sets(info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn update_descriptor_sets(&self, descriptor_writes: &[vk::WriteDescriptorSet]) {
        unsafe { self.device.update_descriptor_sets(descriptor_writes, &[]) }
    }

    pub fn create_sampler(&self, info: &vk::SamplerCreateInfo) -> Result<vk::Sampler, VulkanError> {
        unsafe { self.device.create_sampler(info, None) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn destroy_sampler(&self, sampler: vk::Sampler) {
        unsafe {
            self.device.destroy_sampler(sampler, None);
        }
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), VulkanError> {
        unsafe { self.device.begin_command_buffer(command_buffer, begin_info) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), VulkanError> {
        unsafe { self.device.end_command_buffer(command_buffer) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn cmd_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.device.cmd_pipeline_barrier(
                command_buffer,
                src_stage,
                dst_stage,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn wait_for_fences(&self, fences: &[vk::Fence]) -> Result<(), VulkanError> {
        unsafe { self.device.wait_for_fences(fences, true, FENCE_TIMEOUT) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn reset_fences(&self, fences: &[vk::Fence]) -> Result<(), VulkanError> {
        unsafe { self.device.reset_fences(fences) }
            .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn cmd_begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        info: &vk::RenderPassBeginInfo,
    ) {
        unsafe {
            self.device
                .cmd_begin_render_pass(command_buffer, info, vk::SubpassContents::INLINE);
        }
    }

    pub fn cmd_next_subpass(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device
                .cmd_next_subpass(command_buffer, vk::SubpassContents::INLINE);
        }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe {
            self.device.cmd_end_render_pass(command_buffer);
        }
    }

    pub fn cmd_bind_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        bind: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        unsafe {
            self.device
                .cmd_bind_pipeline(command_buffer, bind, pipeline)
        }
    }

    pub fn cmd_bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        pipeline_bind_point: vk::PipelineBindPoint,
        descriptor_sets: &[vk::DescriptorSet],
    ) {
        unsafe {
            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                pipeline_bind_point,
                pipeline_layout,
                0,
                descriptor_sets,
                &[],
            );
        }
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: vk::CommandBuffer,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        unsafe {
            self.device
                .cmd_bind_vertex_buffers(command_buffer, 0, buffers, offsets);
        }
    }

    pub fn cmd_bind_index_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
    ) {
        unsafe {
            self.device
                .cmd_bind_index_buffer(command_buffer, buffer, offset, vk::IndexType::UINT32)
        }
    }

    pub fn cmd_draw_index(&self, command_buffer: vk::CommandBuffer, index_count: u32) {
        unsafe {
            self.device
                .cmd_draw_indexed(command_buffer, index_count, 1, 0, 0, 0);
        }
    }

    pub fn cmd_copy_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        copy_regions: &[vk::BufferCopy],
    ) {
        unsafe {
            self.device
                .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, copy_regions);
        }
    }

    pub fn cmd_copy_buffer_to_image(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        image: vk::Image,
        image_layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        unsafe {
            self.device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                image_layout,
                regions,
            );
        }
    }

    pub fn map_memory(
        &self,
        memory: vk::DeviceMemory,
        size: vk::DeviceSize,
    ) -> Result<*mut c_void, VulkanError> {
        unsafe {
            self.device
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
        }
        .map_err(|err| VulkanError::DeviceError(err.to_string()))
    }

    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe {
            self.device.unmap_memory(memory);
        }
    }
    
    pub fn cmd_update_buffer(&self, command_buffer: vk::CommandBuffer, buffer: vk::Buffer, data: &[u8]) {
        unsafe {
            self.device.cmd_update_buffer(command_buffer, buffer, 0, data);
        }
    }
}

pub struct VulkanDeviceBuilder<'a> {
    instance: Rc<VulkanInstance>,
    physical_device: &'a PhysicalDevice,
    extensions: Vec<DeviceExtensions>,
    features: Features,
}

impl<'a> VulkanDeviceBuilder<'a> {
    pub fn new(instance: Rc<VulkanInstance>, physical_device: &'a PhysicalDevice) -> Self {
        VulkanDeviceBuilder {
            instance,
            physical_device,
            extensions: vec![],
            features: Features::default(),
        }
    }

    pub fn with_extensions(mut self, extensions: &[DeviceExtensions]) -> Self {
        self.extensions.extend_from_slice(extensions);
        self
    }

    pub fn with_features(mut self, features: Features) -> Self {
        self.features = features;
        self
    }

    pub fn build(self) -> Result<VulkanDevice, VulkanError> {
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.physical_device.get_queue_family())
            .queue_priorities(&[1.0])
            .build();

        let extension_names: Vec<*const c_char> = self
            .extensions
            .iter()
            .map(|extension| extension.name().as_ptr())
            .collect();

        let mut desc_index_features = PhysicalDeviceDescriptorIndexingFeaturesEXT::builder()
            .runtime_descriptor_array(self.features.runtime_descriptor_array)
            .build();

        let required_features = vk::PhysicalDeviceFeatures::builder()
            .geometry_shader(self.features.geometry_shader)
            .sampler_anisotropy(self.features.sampler_anisotropy)
            .tessellation_shader(self.features.tessellation_shader)
            .fragment_stores_and_atomics(self.features.fragment_stores_and_atomics)
            .build();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&[queue_info])
            .enabled_extension_names(&extension_names)
            .enabled_features(&required_features)
            .push_next(&mut desc_index_features)
            .build();

        let device = self
            .instance
            .create_device(self.physical_device.get(), &create_info)?;

        let queue = unsafe { device.get_device_queue(self.physical_device.get_queue_family(), 0) };

        Ok(VulkanDevice {
            instance: self.instance,
            device,
            queue,
        })
    }
}
