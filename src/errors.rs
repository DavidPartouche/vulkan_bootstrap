use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum VulkanError {
    DebugCreationError(String),
    DepthResourcesCreationError(String),
    DeviceError(String),
    ImageCreationError(String),
    InstanceCreationError(String),
    InstanceError(String),
    PipelineError(String),
    PhysicalDeviceCreationError(String),
    RenderPassCreationError(String),
    ShaderCreationError(String),
    SurfaceError(String),
    SwapchainCreationError(String),
    SwapchainError(String),
    TextureCreationError(String),
    VertexBufferCreationError(String),
}

impl Display for VulkanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Vulkan Error: {:?}", self)
    }
}
