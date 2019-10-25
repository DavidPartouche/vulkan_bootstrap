use std::ffi::{CStr, CString};
use std::os::raw::c_void;

use ash::extensions::{ext, khr};
use ash::version::{EntryV1_0, InstanceV1_0, InstanceV1_1};
use ash::vk;

use crate::debug::{DebugOptions, DebugSeverity, DebugType};
use crate::errors::VulkanError;
use crate::extensions::DeviceExtensions;
use semver::Version;

#[derive(Clone)]
pub struct ApplicationInfo {
    pub application_name: String,
    pub application_version: Version,
    pub engine_name: String,
    pub engine_version: Version,
}

impl Default for ApplicationInfo {
    fn default() -> Self {
        ApplicationInfo {
            application_name: String::new(),
            application_version: Version::new(0, 0, 0),
            engine_name: String::new(),
            engine_version: Version::new(0, 0, 0),
        }
    }
}

pub struct VulkanInstance {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
    messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_utils) = &self.debug_utils {
                debug_utils.destroy_debug_utils_messenger(self.messenger.unwrap(), None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanInstance {
    pub fn get(&self) -> &ash::Instance {
        &self.instance
    }

    pub fn create_win_32_surface(
        &self,
        hinstance: vk::HINSTANCE,
        hwnd: vk::HWND,
    ) -> Result<(khr::Surface, vk::SurfaceKHR), VulkanError> {
        let create_info = vk::Win32SurfaceCreateInfoKHR::builder()
            .hinstance(hinstance)
            .hwnd(hwnd)
            .build();

        let surface_loader = khr::Surface::new(&self.entry, &self.instance);

        let win32_surface_loader = khr::Win32Surface::new(&self.entry, &self.instance);

        let surface = unsafe { win32_surface_loader.create_win32_surface(&create_info, None) }
            .map_err(|err| VulkanError::InstanceError(err.to_string()))?;

        Ok((surface_loader, surface))
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<vk::PhysicalDevice>, VulkanError> {
        Ok(unsafe { self.instance.enumerate_physical_devices() }
            .map_err(|err| VulkanError::InstanceError(err.to_string()))?)
    }

    pub fn get_physical_device_queue_family_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .get_physical_device_queue_family_properties(physical_device)
        }
    }

    pub fn enumerate_device_extension_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<DeviceExtensions>, VulkanError> {
        Ok(unsafe {
            self.instance
                .enumerate_device_extension_properties(physical_device)
        }
        .map_err(|err| VulkanError::InstanceError(err.to_string()))?
        .iter()
        .map(|property| {
            let name = unsafe { CStr::from_ptr(property.extension_name.as_ptr()) };
            DeviceExtensions::from(name.to_str().unwrap())
        })
        .collect())
    }

    pub fn get_physical_device_features(
        &self,
        device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceFeatures {
        unsafe { self.instance.get_physical_device_features(device) }
    }

    pub fn get_physical_device_memory_properties(
        &self,
        device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceMemoryProperties {
        unsafe { self.instance.get_physical_device_memory_properties(device) }
    }

    pub fn get_physical_device_properties2(
        &self,
        device: vk::PhysicalDevice,
        prop: &mut vk::PhysicalDeviceProperties2,
    ) -> vk::PhysicalDeviceProperties2 {
        unsafe {
            self.instance.get_physical_device_properties2(device, prop);
            *prop
        }
    }

    pub fn get_physical_device_format_properties(
        &self,
        device: vk::PhysicalDevice,
        format: vk::Format,
    ) -> vk::FormatProperties {
        unsafe {
            self.instance
                .get_physical_device_format_properties(device, format)
        }
    }

    pub fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo,
    ) -> Result<ash::Device, VulkanError> {
        unsafe {
            self.instance
                .create_device(physical_device, create_info, None)
        }
        .map_err(|err| VulkanError::InstanceError(err.to_string()))
    }

    pub fn find_memory_type(
        &self,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let memory_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(physical_device)
        };

        memory_properties
            .memory_types
            .iter()
            .enumerate()
            .find_map(|(index, memory_type)| {
                if type_filter & (1 << index as u32) != 0
                    && memory_type.property_flags.contains(properties)
                {
                    Some(index as u32)
                } else {
                    None
                }
            })
    }

    unsafe extern "system" fn vulkan_debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        ty: vk::DebugUtilsMessageTypeFlagsEXT,
        callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut c_void,
    ) -> u32 {
        let message = CStr::from_ptr((*callback_data).p_message);

        let message = if ty.contains(vk::DebugUtilsMessageTypeFlagsEXT::GENERAL) {
            format!("General Layer: {:?}", message)
        } else if ty.contains(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION) {
            format!("Validation layer: {:?}", message)
        } else {
            format!("Performance Layer: {:?}", message)
        };

        if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE) {
            log::trace!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::INFO) {
            log::info!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING) {
            log::warn!("{}", message);
        } else if severity.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR) {
            log::error!("{}", message);
        }

        vk::FALSE
    }
}

pub struct VulkanInstanceBuilder<'a> {
    debug_options: DebugOptions,
    application_info: Option<&'a ApplicationInfo>,
}

impl<'a> VulkanInstanceBuilder<'a> {
    pub fn new() -> Self {
        VulkanInstanceBuilder {
            debug_options: DebugOptions::default(),
            application_info: None,
        }
    }

    pub fn with_debug_options(mut self, debug_options: DebugOptions) -> Self {
        self.debug_options = debug_options;
        self
    }

    pub fn with_application_info(mut self, application_info: &'a ApplicationInfo) -> Self {
        self.application_info = Some(application_info);
        self
    }

    pub fn build(self) -> Result<VulkanInstance, VulkanError> {
        let application_info = self.application_info.unwrap();

        let application_version = ash::vk_make_version!(
            application_info.application_version.major,
            application_info.application_version.minor,
            application_info.application_version.patch
        );
        let engine_version = ash::vk_make_version!(
            application_info.application_version.major,
            application_info.application_version.minor,
            application_info.application_version.patch
        );
        let api_version = ash::vk_make_version!(1, 1, 0);

        let application_name = CString::new(application_info.application_name.as_bytes()).unwrap();
        let engine_name = CString::new(application_info.engine_name.as_bytes()).unwrap();

        let application_info = vk::ApplicationInfo::builder()
            .application_name(
                CStr::from_bytes_with_nul(application_name.as_bytes_with_nul()).unwrap(),
            )
            .application_version(application_version)
            .engine_name(CStr::from_bytes_with_nul(engine_name.as_bytes_with_nul()).unwrap())
            .engine_version(engine_version)
            .api_version(api_version)
            .build();

        let mut layers = vec![];
        let mut extensions = vec![
            khr::Surface::name().as_ptr(),
            khr::Win32Surface::name().as_ptr(),
        ];

        let debug_enabled = self.debug_options.debug_type != DebugType::none()
            && self.debug_options.debug_severity != DebugSeverity::none();

        if debug_enabled {
            let debug_layer = CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap();
            layers.push(debug_layer.as_ptr());
            extensions.push(ext::DebugUtils::name().as_ptr())
        }

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(layers.as_slice())
            .enabled_extension_names(extensions.as_slice())
            .build();

        let entry =
            ash::Entry::new().map_err(|err| VulkanError::InstanceCreationError(err.to_string()))?;
        let instance = unsafe { entry.create_instance(&create_info, None) }
            .map_err(|err| VulkanError::InstanceCreationError(err.to_string()))?;

        let (debug_utils, messenger) = if debug_enabled {
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(self.debug_options.debug_severity.into())
                .message_type(self.debug_options.debug_type.into())
                .pfn_user_callback(Some(VulkanInstance::vulkan_debug_callback))
                .build();

            let debug_utils = Some(ext::DebugUtils::new(&entry, &instance));
            let messenger = Some(
                unsafe {
                    debug_utils
                        .as_ref()
                        .unwrap()
                        .create_debug_utils_messenger(&debug_info, None)
                }
                .map_err(|err| VulkanError::DebugCreationError(err.to_string()))?,
            );
            (debug_utils, messenger)
        } else {
            (None, None)
        };

        Ok(VulkanInstance {
            entry,
            instance,
            debug_utils,
            messenger,
        })
    }
}
