use ash::extensions::khr;
use ash::vk;

use crate::errors::VulkanError;
use crate::instance::VulkanInstance;
use crate::windows::Win32Window;

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct Surface {
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

impl Surface {
    pub fn get(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn get_physical_device_surface_support(
        &self,
        device: vk::PhysicalDevice,
        index: u32,
    ) -> bool {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_support(device, index, self.surface)
        }
    }

    pub fn query_swapchain_support(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<SwapchainSupportDetails, VulkanError> {
        let capabilities = self.get_physical_device_surface_capabilities(device)?;
        let formats = self.get_physical_device_surface_formats(device)?;
        let present_modes = self.get_physical_device_surface_present_modes(device)?;

        Ok(SwapchainSupportDetails {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn get_physical_device_surface_capabilities(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, VulkanError> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_capabilities(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))
    }

    pub fn get_physical_device_surface_formats(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, VulkanError> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_formats(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))
    }

    pub fn get_physical_device_surface_present_modes(
        &self,
        device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>, VulkanError> {
        unsafe {
            self.surface_loader
                .get_physical_device_surface_present_modes(device, self.surface)
        }
        .map_err(|err| VulkanError::SurfaceError(err.to_string()))
    }
}

pub struct SurfaceBuilder<'a> {
    instance: &'a VulkanInstance,
    window: Win32Window,
}

impl<'a> SurfaceBuilder<'a> {
    pub fn new(instance: &'a VulkanInstance) -> Self {
        SurfaceBuilder {
            instance,
            window: Win32Window::default(),
        }
    }

    pub fn with_window(mut self, window: Win32Window) -> Self {
        self.window = window;
        self
    }

    pub fn build(self) -> Result<Surface, VulkanError> {
        let (surface_loader, surface) = self
            .instance
            .create_win_32_surface(self.window.hinstance, self.window.hwnd)?;

        Ok(Surface {
            surface_loader,
            surface,
        })
    }
}
