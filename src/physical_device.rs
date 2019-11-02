use ash::vk;

use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use crate::features::Features;
use crate::instance::VulkanInstance;
use crate::surface::Surface;
use std::rc::Rc;

pub struct PhysicalDevice {
    instance: Rc<VulkanInstance>,
    physical_device: vk::PhysicalDevice,
    queue_family: u32,
}

impl PhysicalDevice {
    pub fn get(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_queue_family(&self) -> u32 {
        self.queue_family
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
    features: Features,
}

impl<'a> PhysicalDeviceBuilder<'a> {
    pub fn new(instance: Rc<VulkanInstance>, surface: &'a Surface) -> Self {
        PhysicalDeviceBuilder {
            instance,
            surface,
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

    pub fn build(self) -> Result<PhysicalDevice, VulkanError> {
        let physical_devices = self.instance.enumerate_physical_devices()?;

        let (physical_device, queue_family) = physical_devices
            .into_iter()
            .find_map(|device| {
                let queue_family = self.find_queue_family(device);
                if self.is_device_suitable(device) && queue_family.is_some() {
                    Some((device, queue_family.unwrap()))
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
            queue_family,
        })
    }

    fn is_device_suitable(&self, device: vk::PhysicalDevice) -> bool {
        let swapchain_support = self.surface.query_swapchain_support(device).unwrap();

        self.check_device_extensions_support(device)
            && self.check_device_features_support(device)
            && !swapchain_support.formats.is_empty()
            && !swapchain_support.present_modes.is_empty()
    }

    fn find_queue_family(&self, device: vk::PhysicalDevice) -> Option<u32> {
        let queue_families = self
            .instance
            .get_physical_device_queue_family_properties(device);

        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && self
                    .surface
                    .get_physical_device_surface_support(device, index as u32)
            {
                return Some(index as u32);
            }
        }
        None
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

    fn check_device_features_support(&self, device: vk::PhysicalDevice) -> bool {
        let available_features = self.instance.get_physical_device_features(device);

        (!self.features.geometry_shader || available_features.geometry_shader == vk::TRUE)
            && (!self.features.sampler_anisotropy
                || available_features.sampler_anisotropy == vk::TRUE)
            && (!self.features.tessellation_shader
                || available_features.tessellation_shader == vk::TRUE)
            && (!self.features.fragment_stores_and_atomics
                || available_features.fragment_stores_and_atomics == vk::TRUE)
    }
}
