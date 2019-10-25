use std::os::raw::c_void;
use std::ptr::null;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::command_buffers::{CommandBuffers, CommandBuffersBuilder};
use crate::debug::{DebugSeverity, DebugType};
use crate::depth_resources::{DepthResources, DepthResourcesBuilder};
use crate::device::{VulkanDevice, VulkanDeviceBuilder};
use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::frame_buffer::{FrameBuffers, FrameBuffersBuilder};
use crate::instance::{ApplicationInfo, VulkanInstance, VulkanInstanceBuilder};
use crate::physical_device::{PhysicalDevice, PhysicalDeviceBuilder};
use crate::render_pass::{RenderPass, RenderPassBuilder};
use crate::surface::{Surface, SurfaceBuilder};
use crate::swapchain::{Swapchain, SwapchainBuilder};

pub struct VulkanContext {
    frame_buffers: FrameBuffers,
    render_pass: RenderPass,
    _depth_resources: DepthResources,
    swapchain: Swapchain,
    command_buffers: CommandBuffers,
    device: Rc<VulkanDevice>,
    physical_device: PhysicalDevice,
    _surface: Surface,
    instance: Rc<VulkanInstance>,
    frame_index: usize,
    frames_count: usize,
    back_buffer_index: usize,
    clear_value: [f32; 4],
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.device.graphics_queue_wait_idle().unwrap();
        self.device.present_queue_wait_idle().unwrap();
    }
}

impl VulkanContext {
    pub fn get_instance(&self) -> &Rc<VulkanInstance> {
        &self.instance
    }

    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn get_device(&self) -> &Rc<VulkanDevice> {
        &self.device
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn get_command_buffers(&self) -> &CommandBuffers {
        &self.command_buffers
    }

    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers.get(self.frame_index)
    }

    pub fn get_current_back_buffer(&self) -> vk::Image {
        self.swapchain.get_image(self.back_buffer_index)
    }

    pub fn get_current_back_buffer_view(&self) -> vk::ImageView {
        self.swapchain.get_image_view(self.back_buffer_index)
    }

    pub fn set_clear_value(&mut self, clear_value: [f32; 4]) {
        self.clear_value = clear_value;
    }

    pub fn create_buffer(
        &self,
        ty: BufferType,
        size: vk::DeviceSize,
        data: *const c_void,
    ) -> Result<Buffer, VulkanError> {
        let staging_buffer = BufferBuilder::new(self)
            .with_type(BufferType::Staging)
            .with_size(size)
            .build()?;

        staging_buffer.copy_data(data)?;

        let buffer = BufferBuilder::new(self)
            .with_type(ty)
            .with_size(size)
            .build()?;

        self.command_buffers
            .copy_buffer(staging_buffer.get(), buffer.get(), size)?;

        Ok(buffer)
    }

    pub fn frame_begin(&mut self) -> Result<(), VulkanError> {
        self.command_buffers.wait_for_fence(self.frame_index)?;

        self.back_buffer_index = self.swapchain.acquire_next_image(
            self.command_buffers
                .get_present_complete_semaphore(self.frame_index),
        )?;

        self.command_buffers.begin_command_buffer(self.frame_index)
    }

    pub fn frame_end(&self) -> Result<(), VulkanError> {
        self.command_buffers.end_command_buffer(self.frame_index)?;
        self.command_buffers.reset_fence(self.frame_index)?;
        self.command_buffers.queue_submit(self.frame_index)
    }

    pub fn frame_present(&mut self) -> Result<(), VulkanError> {
        self.swapchain.queue_present(
            self.command_buffers
                .get_render_complete_semaphore(self.frame_index),
            self.back_buffer_index as u32,
        )?;
        self.frame_index = (self.frame_index + 1) % self.frames_count;
        Ok(())
    }

    pub fn begin_render_pass(&self) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: self.clear_value,
            },
        };
        let clear_depth = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue::builder()
                .depth(1.0)
                .stencil(0)
                .build(),
        };
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.get())
            .framebuffer(self.frame_buffers.get(self.back_buffer_index))
            .render_area(
                vk::Rect2D::builder()
                    .extent(self.swapchain.get_extent())
                    .build(),
            )
            .clear_values(&[clear_color, clear_depth])
            .build();

        self.device
            .cmd_begin_render_pass(self.command_buffers.get(self.frame_index), &info);
    }
    pub fn end_render_pass(&self) {
        self.device
            .cmd_end_render_pass(self.command_buffers.get(self.frame_index));
    }

    pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer, VulkanError> {
        self.command_buffers.begin_single_time_commands()
    }

    pub fn end_single_time_commands(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), VulkanError> {
        self.command_buffers
            .end_single_time_commands(command_buffer)
    }
}

pub struct VulkanContextBuilder<'a> {
    debug: bool,
    debug_severity: DebugSeverity,
    debug_type: DebugType,
    hinstance: *const c_void,
    hwnd: *const c_void,
    width: u32,
    height: u32,
    extensions: Vec<DeviceExtensions>,
    frames_count: u32,
    application_info: Option<&'a ApplicationInfo>,
    sampler_anisotropy: bool,
    runtime_descriptor_array: bool,
}

impl<'a> Default for VulkanContextBuilder<'a> {
    fn default() -> Self {
        VulkanContextBuilder {
            debug: false,
            debug_severity: DebugSeverity::default(),
            debug_type: DebugType::default(),
            hinstance: null(),
            hwnd: null(),
            width: 0,
            height: 0,
            extensions: vec![],
            frames_count: 2,
            application_info: None,
            sampler_anisotropy: false,
            runtime_descriptor_array: false,
        }
    }
}

impl<'a> VulkanContextBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debug_enabled(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_debug_severity(mut self, debug_severity: DebugSeverity) -> Self {
        self.debug_severity = debug_severity;
        self
    }

    pub fn with_debug_type(mut self, debug_type: DebugType) -> Self {
        self.debug_type = debug_type;
        self
    }

    pub fn with_hinstance(mut self, hinstance: *const c_void) -> Self {
        self.hinstance = hinstance;
        self
    }

    pub fn with_hwnd(mut self, hwnd: *const c_void) -> Self {
        self.hwnd = hwnd;
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

    pub fn with_extensions(mut self, extensions: Vec<DeviceExtensions>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_frames_count(mut self, frames_count: u32) -> Self {
        self.frames_count = frames_count;
        self
    }

    pub fn with_application_name(mut self, application_info: &'a ApplicationInfo) -> Self {
        self.application_info = Some(application_info);
        self
    }

    pub fn with_sampler_anisotropy(mut self, sampler_anisotropy: bool) -> Self {
        self.sampler_anisotropy = sampler_anisotropy;
        self
    }

    pub fn with_runtime_descriptor_array(mut self, runtime_descriptor_array: bool) -> Self {
        self.runtime_descriptor_array = runtime_descriptor_array;
        self
    }

    pub fn build(self) -> Result<VulkanContext, VulkanError> {
        let instance = Rc::new(self.create_instance()?);

        let surface = self.create_surface(&instance)?;

        let physical_device = self.select_physical_device(Rc::clone(&instance), &surface)?;

        let device = Rc::new(self.create_logical_device(Rc::clone(&instance), &physical_device)?);

        let command_buffers = self.create_command_buffers(&physical_device, Rc::clone(&device))?;

        let swapchain = self.create_swapchain(Rc::clone(&device), &surface, &physical_device)?;

        let depth_resources = self.create_depth_resources(
            &instance,
            &physical_device,
            Rc::clone(&device),
            &command_buffers,
        )?;

        let render_pass =
            self.create_render_pass(Rc::clone(&device), &swapchain, &depth_resources)?;

        let frame_buffers = self.create_frame_buffers(
            Rc::clone(&device),
            &render_pass,
            &swapchain,
            &depth_resources,
        )?;

        Ok(VulkanContext {
            instance,
            _surface: surface,
            physical_device,
            device,
            command_buffers,
            swapchain,
            _depth_resources: depth_resources,
            render_pass,
            frame_buffers,
            frame_index: 0,
            frames_count: self.frames_count as usize,
            back_buffer_index: 0,
            clear_value: [1.0, 1.0, 1.0, 1.0],
        })
    }

    fn create_instance(&self) -> Result<VulkanInstance, VulkanError> {
        VulkanInstanceBuilder::new()
            .with_debug_enabled(self.debug)
            .with_debug_severity(self.debug_severity)
            .with_debug_type(self.debug_type)
            .with_application_info(self.application_info)
            .build()
    }

    fn create_surface(&self, instance: &VulkanInstance) -> Result<Surface, VulkanError> {
        SurfaceBuilder::new(instance)
            .with_hinstance(self.hinstance)
            .with_hwnd(self.hwnd)
            .build()
    }

    fn select_physical_device(
        &self,
        instance: Rc<VulkanInstance>,
        surface: &Surface,
    ) -> Result<PhysicalDevice, VulkanError> {
        PhysicalDeviceBuilder::new(instance, surface)
            .with_extensions(&self.extensions)
            .with_sampler_anisotropy(self.sampler_anisotropy)
            .build()
    }

    fn create_logical_device(
        &self,
        instance: Rc<VulkanInstance>,
        physical_device: &PhysicalDevice,
    ) -> Result<VulkanDevice, VulkanError> {
        VulkanDeviceBuilder::new(instance, physical_device)
            .with_extensions(&self.extensions)
            .with_sampler_anisotropy(self.sampler_anisotropy)
            .with_runtime_descriptor_array(self.runtime_descriptor_array)
            .build()
    }

    fn create_command_buffers(
        &self,
        physical_device: &PhysicalDevice,
        device: Rc<VulkanDevice>,
    ) -> Result<CommandBuffers, VulkanError> {
        CommandBuffersBuilder::new(physical_device, device)
            .with_buffer_count(self.frames_count)
            .build()
    }

    fn create_swapchain(
        &self,
        device: Rc<VulkanDevice>,
        surface: &Surface,
        physical_device: &PhysicalDevice,
    ) -> Result<Swapchain, VulkanError> {
        SwapchainBuilder::new(device, surface, physical_device)
            .with_width(self.width)
            .with_height(self.height)
            .with_frames_count(self.frames_count)
            .build()
    }

    fn create_depth_resources(
        &self,
        instance: &VulkanInstance,
        physical_device: &PhysicalDevice,
        device: Rc<VulkanDevice>,
        command_buffers: &CommandBuffers,
    ) -> Result<DepthResources, VulkanError> {
        DepthResourcesBuilder::new(instance, physical_device, device, command_buffers)
            .with_width(self.width)
            .with_height(self.height)
            .build()
    }

    fn create_render_pass(
        &self,
        device: Rc<VulkanDevice>,
        swapchain: &Swapchain,
        depth_resources: &DepthResources,
    ) -> Result<RenderPass, VulkanError> {
        RenderPassBuilder::new(device, swapchain, depth_resources).build()
    }

    fn create_frame_buffers(
        &self,
        device: Rc<VulkanDevice>,
        render_pass: &RenderPass,
        swapchain: &Swapchain,
        depth_resources: &DepthResources,
    ) -> Result<FrameBuffers, VulkanError> {
        FrameBuffersBuilder::new(device, render_pass, swapchain, depth_resources)
            .with_width(self.width)
            .with_height(self.height)
            .with_frames_count(self.frames_count)
            .build()
    }
}
