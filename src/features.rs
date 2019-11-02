#[derive(Default, Copy, Clone)]
pub struct Features {
    pub geometry_shader: bool,
    pub tessellation_shader: bool,
    pub runtime_descriptor_array: bool,
    pub sampler_anisotropy: bool,
    pub fragment_stores_and_atomics: bool,
}

impl Features {
    pub fn none() -> Self {
        Features::default()
    }

    pub fn all() -> Self {
        Features {
            geometry_shader: true,
            tessellation_shader: true,
            runtime_descriptor_array: true,
            sampler_anisotropy: true,
            fragment_stores_and_atomics: true,
        }
    }
}
