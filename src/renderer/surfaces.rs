use std::ffi::{CStr, CString};
use ash::extensions::khr::{Surface, Swapchain, Win32Surface};
use ash::vk::{PhysicalDevice, SurfaceKHR};
use ash::Instance;

pub struct Surfaces {
    pub surface: Surface,
    pub surface_khr: SurfaceKHR,
}

impl Surfaces {
    fn new() {
        
    }

    pub fn require_surface_extension_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }
}