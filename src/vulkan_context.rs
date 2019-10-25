use std::os::raw::c_void;
use std::rc::Rc;

use ash::vk;

use crate::buffer::{Buffer, BufferBuilder, BufferType};
use crate::command_buffers::{CommandBuffers, CommandBuffersBuilder};
use crate::debug::DebugOptions;
use crate::depth_resources::{DepthResources, DepthResourcesBuilder};
use crate::device::{VulkanDevice, VulkanDeviceBuilder};
use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::features::Features;
use crate::frame_buffer::{FrameBuffers, FrameBuffersBuilder};
use crate::instance::{ApplicationInfo, VulkanInstance, VulkanInstanceBuilder};
use crate::physical_device::{PhysicalDevice, PhysicalDeviceBuilder};
use crate::render_pass::{RenderPass, RenderPassBuilder};
use crate::surface::{Surface, SurfaceBuilder};
use crate::swapchain::{Swapchain, SwapchainBuilder};
use crate::windows::Win32Window;
use std::mem;

pub struct VulkanContext {
    frame_buffers: Option<FrameBuffers>,
    render_pass: Option<RenderPass>,
    depth_resources: Option<DepthResources>,
    swapchain: Option<Swapchain>,
    command_buffers: CommandBuffers,
    device: Rc<VulkanDevice>,
    physical_device: PhysicalDevice,
    surface: Surface,
    instance: Rc<VulkanInstance>,
    frame_index: usize,
    frames_count: usize,
    back_buffer_index: usize,
    clear_value: [f32; 4],
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.device.queue_wait_idle().unwrap();
    }
}

impl VulkanContext {
    pub fn get_instance(&self) -> &Rc<VulkanInstance> {
        &self.instance
    }

    pub fn get_surface(&self) -> &Surface {
        &self.surface
    }

    pub fn get_physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    pub fn get_device(&self) -> &Rc<VulkanDevice> {
        &self.device
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        self.swapchain.as_ref().unwrap()
    }

    pub fn get_depth_resources(&self) -> &DepthResources {
        self.depth_resources.as_ref().unwrap()
    }

    pub fn get_render_pass(&self) -> &RenderPass {
        self.render_pass.as_ref().unwrap()
    }

    pub fn get_current_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers.get(self.frame_index)
    }

    pub fn get_current_back_buffer(&self) -> vk::Image {
        self.swapchain
            .as_ref()
            .unwrap()
            .get_image(self.back_buffer_index)
    }

    pub fn get_current_back_buffer_view(&self) -> vk::ImageView {
        self.swapchain
            .as_ref()
            .unwrap()
            .get_image_view(self.back_buffer_index)
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

        self.back_buffer_index = self.swapchain.as_ref().unwrap().acquire_next_image(
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
        self.swapchain.as_ref().unwrap().queue_present(
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
            .render_pass(self.render_pass.as_ref().unwrap().get())
            .framebuffer(
                self.frame_buffers
                    .as_ref()
                    .unwrap()
                    .get(self.back_buffer_index),
            )
            .render_area(
                vk::Rect2D::builder()
                    .extent(self.swapchain.as_ref().unwrap().get_extent())
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

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), VulkanError> {
        self.device.queue_wait_idle()?;

        if let Some(frame_buffers) = self.frame_buffers.take() {
            mem::drop(frame_buffers);
        }

        if let Some(render_pass) = self.render_pass.take() {
            mem::drop(render_pass);
        }

        if let Some(depth_resources) = self.depth_resources.take() {
            mem::drop(depth_resources);
        }

        let old_swapchain = self.swapchain.take();
        self.swapchain = Some(self.create_swapchain(old_swapchain, width, height)?);

        self.depth_resources = Some(self.create_depth_resources(width, height)?);

        self.render_pass = Some(self.create_render_pass()?);

        self.frame_buffers = Some(self.create_frame_buffers(width, height)?);

        Ok(())
    }

    fn create_swapchain(
        &mut self,
        old_swapchain: Option<Swapchain>,
        width: u32,
        height: u32,
    ) -> Result<Swapchain, VulkanError> {
        SwapchainBuilder::new(self)
            .with_old_swapchain(old_swapchain)
            .with_width(width)
            .with_height(height)
            .with_frames_count(self.frames_count as u32)
            .build()
    }

    fn create_depth_resources(
        &self,
        width: u32,
        height: u32,
    ) -> Result<DepthResources, VulkanError> {
        DepthResourcesBuilder::new(self)
            .with_width(width)
            .with_height(height)
            .build()
    }

    fn create_render_pass(&self) -> Result<RenderPass, VulkanError> {
        RenderPassBuilder::new(self).build()
    }

    fn create_frame_buffers(&self, width: u32, height: u32) -> Result<FrameBuffers, VulkanError> {
        FrameBuffersBuilder::new(self)
            .with_width(width)
            .with_height(height)
            .with_frames_count(self.frames_count as u32)
            .build()
    }
}

pub struct VulkanContextBuilder {
    application_info: ApplicationInfo,
    debug_options: DebugOptions,
    window: Win32Window,
    extensions: Vec<DeviceExtensions>,
    features: Features,
    frames_count: u32,
}

impl Default for VulkanContextBuilder {
    fn default() -> Self {
        VulkanContextBuilder {
            application_info: ApplicationInfo::default(),
            debug_options: DebugOptions::default(),
            window: Win32Window::default(),
            features: Features::default(),
            extensions: vec![],
            frames_count: 2,
        }
    }
}

impl VulkanContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_application_info(mut self, application_info: ApplicationInfo) -> Self {
        self.application_info = application_info.clone();
        self
    }

    pub fn with_debug_options(mut self, debug_options: DebugOptions) -> Self {
        self.debug_options = debug_options;
        self
    }

    pub fn with_window(mut self, window: Win32Window) -> Self {
        self.window = window;
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<DeviceExtensions>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_features(mut self, features: Features) -> Self {
        self.features = features;
        self
    }

    pub fn with_frames_count(mut self, frames_count: u32) -> Self {
        self.frames_count = frames_count;
        self
    }

    pub fn build(self) -> Result<VulkanContext, VulkanError> {
        let instance = Rc::new(self.create_instance()?);

        let surface = self.create_surface(&instance)?;

        let physical_device = self.select_physical_device(Rc::clone(&instance), &surface)?;

        let device = Rc::new(self.create_logical_device(Rc::clone(&instance), &physical_device)?);

        let command_buffers = self.create_command_buffers(&physical_device, Rc::clone(&device))?;

        let mut context = VulkanContext {
            instance,
            surface,
            physical_device,
            device,
            command_buffers,
            swapchain: None,
            depth_resources: None,
            render_pass: None,
            frame_buffers: None,
            frame_index: 0,
            frames_count: self.frames_count as usize,
            back_buffer_index: 0,
            clear_value: [1.0, 1.0, 1.0, 1.0],
        };

        context.resize(self.window.width, self.window.height)?;

        Ok(context)
    }

    fn create_instance(&self) -> Result<VulkanInstance, VulkanError> {
        VulkanInstanceBuilder::new()
            .with_debug_options(self.debug_options)
            .with_application_info(&self.application_info)
            .build()
    }

    fn create_surface(&self, instance: &VulkanInstance) -> Result<Surface, VulkanError> {
        SurfaceBuilder::new(instance)
            .with_window(self.window)
            .build()
    }

    fn select_physical_device(
        &self,
        instance: Rc<VulkanInstance>,
        surface: &Surface,
    ) -> Result<PhysicalDevice, VulkanError> {
        PhysicalDeviceBuilder::new(instance, surface)
            .with_extensions(&self.extensions)
            .with_features(self.features)
            .build()
    }

    fn create_logical_device(
        &self,
        instance: Rc<VulkanInstance>,
        physical_device: &PhysicalDevice,
    ) -> Result<VulkanDevice, VulkanError> {
        VulkanDeviceBuilder::new(instance, physical_device)
            .with_extensions(&self.extensions)
            .with_features(self.features)
            .build()
    }

    fn create_command_buffers(
        &self,
        physical_device: &PhysicalDevice,
        device: Rc<VulkanDevice>,
    ) -> Result<CommandBuffers, VulkanError> {
        CommandBuffersBuilder::new(physical_device, device)
            .with_frames_count(self.frames_count)
            .build()
    }
}
