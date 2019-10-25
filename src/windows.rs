use std::os::raw::c_void;
use std::ptr::null;

#[derive(Copy, Clone)]
pub struct Win32Window {
    pub hinstance: *const c_void,
    pub hwnd: *const c_void,
    pub width: u32,
    pub height: u32,
}

impl Default for Win32Window {
    fn default() -> Self {
        Win32Window {
            hinstance: null(),
            hwnd: null(),
            width: 0,
            height: 0,
        }
    }
}