use ash::vk;

#[derive(Default, Copy, Clone)]
pub struct DebugOptions {
    pub debug_severity: DebugSeverity,
    pub debug_type: DebugType,
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct DebugSeverity {
    pub verbose: bool,
    pub info: bool,
    pub warning: bool,
    pub error: bool,
}

impl DebugSeverity {
    pub fn all() -> Self {
        DebugSeverity {
            verbose: true,
            info: true,
            warning: true,
            error: true,
        }
    }
    
    pub fn none() -> Self {
        DebugSeverity::default()
    }
}

impl Into<vk::DebugUtilsMessageSeverityFlagsEXT> for DebugSeverity {
    fn into(self) -> vk::DebugUtilsMessageSeverityFlagsEXT {
        let mut message_severity = vk::DebugUtilsMessageSeverityFlagsEXT::empty();
        if self.verbose {
            message_severity |= vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE;
        }
        if self.info {
            message_severity |= vk::DebugUtilsMessageSeverityFlagsEXT::INFO;
        }
        if self.warning {
            message_severity |= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
        }
        if self.error {
            message_severity |= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
        }
        message_severity
    }
}

#[derive(Default, Copy, Clone, PartialEq)]
pub struct DebugType {
    pub general: bool,
    pub validation: bool,
    pub performance: bool,
}

impl DebugType {
    pub fn all() -> Self {
        DebugType {
            general: true,
            validation: true,
            performance: true,
        }
    }
    
    pub fn none() -> Self {
        DebugType::default()
    }
}

impl Into<vk::DebugUtilsMessageTypeFlagsEXT> for DebugType {
    fn into(self) -> vk::DebugUtilsMessageTypeFlagsEXT {
        let mut message_type = vk::DebugUtilsMessageTypeFlagsEXT::empty();
        if self.general {
            message_type |= vk::DebugUtilsMessageTypeFlagsEXT::GENERAL;
        }
        if self.performance {
            message_type |= vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE;
        }
        if self.validation {
            message_type |= vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION;
        }
        message_type
    }
}
