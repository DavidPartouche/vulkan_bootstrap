use ash::vk;

use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::instance::VulkanInstance;
use crate::surface::Surface;
use std::rc::Rc;

struct QueueFamilyIndices {
    pub graphics_family: Option<usize>,
    pub present_family: Option<usize>,
}

impl QueueFamilyIndices {
    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct PhysicalDevice {
    instance: Rc<VulkanInstance>,
    physical_device: vk::PhysicalDevice,
    graphics_queue_family: u32,
    present_queue_family: u32,
}

impl PhysicalDevice {
    pub fn get(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_graphics_queue_family(&self) -> u32 {
        self.graphics_queue_family
    }

    pub fn get_present_queue_family(&self) -> u32 {
        self.present_queue_family
    }

    pub fn find_memory_type(
        &self,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let mem_properties = self
            .instance
            .get_physical_device_memory_properties(self.physical_device);

        for i in 0..mem_properties.memory_type_count {
            if type_filter & (1 << i) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Some(i);
            }
        }

        None
    }
}

pub struct PhysicalDeviceBuilder<'a> {
    instance: Rc<VulkanInstance>,
    surface: &'a Surface,
    extensions: Vec<DeviceExtensions>,
    sampler_anisotropy: bool,
}

impl<'a> PhysicalDeviceBuilder<'a> {
    pub fn new(instance: Rc<VulkanInstance>, surface: &'a Surface) -> Self {
        PhysicalDeviceBuilder {
            instance,
            surface,
            extensions: vec![],
            sampler_anisotropy: false,
        }
    }

    pub fn with_extensions(mut self, extensions: &[DeviceExtensions]) -> Self {
        self.extensions.extend_from_slice(extensions);
        self
    }

    pub fn with_sampler_anisotropy(mut self, sampler_anisotropy: bool) -> Self {
        self.sampler_anisotropy = sampler_anisotropy;
        self
    }

    pub fn build(self) -> Result<PhysicalDevice, VulkanError> {
        let physical_devices = self.instance.enumerate_physical_devices()?;

        let (physical_device, queue_family) = physical_devices
            .into_iter()
            .find_map(|device| {
                let queue_family_indices = self.find_queue_families(device);
                if self.is_device_suitable(device) && queue_family_indices.is_complete() {
                    Some((device, queue_family_indices))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                VulkanError::PhysicalDeviceCreationError(String::from(
                    "Cannot find suitable physical device",
                ))
            })?;

        Ok(PhysicalDevice {
            instance: self.instance,
            physical_device,
            graphics_queue_family: queue_family.graphics_family.unwrap() as u32,
            present_queue_family: queue_family.present_family.unwrap() as u32,
        })
    }

    fn is_device_suitable(&self, device: vk::PhysicalDevice) -> bool {
        let swapchain_support = self.surface.query_swapchain_support(device).unwrap();
        let sampler_anisotropy = if self.sampler_anisotropy {
            self.instance
                .get_physical_device_features(device)
                .sampler_anisotropy
                == vk::TRUE
        } else {
            true
        };

        self.check_device_extensions_support(device)
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty()
            && sampler_anisotropy
    }

    fn find_queue_families(&self, device: vk::PhysicalDevice) -> QueueFamilyIndices {
        let queue_families = self
            .instance
            .get_physical_device_queue_family_properties(device);

        let mut graphics_family = None;
        let mut present_family = None;

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics_family = Some(index);
            }

            if self
                .surface
                .get_physical_device_surface_support(device, index as u32)
            {
                present_family = Some(index);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        QueueFamilyIndices {
            graphics_family,
            present_family,
        }
    }

    fn check_device_extensions_support(&self, device: vk::PhysicalDevice) -> bool {
        let available_extensions = self
            .instance
            .enumerate_device_extension_properties(device)
            .unwrap();

        for extension in self.extensions.iter() {
            if available_extensions
                .iter()
                .find(|available_extension| *available_extension == extension)
                .is_none()
            {
                return false;
            }
        }

        true
    }
}
