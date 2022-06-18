use std::ffi::{CStr, CString};
use ash::extensions::khr::{Surface, Swapchain, Win32Surface};
use ash::vk::{PhysicalDevice, SurfaceKHR};
use ash::{Entry, Instance};
use log::info;
use tobj::LoadError::NormalParseError;
use winit::window::Window;
use crate::window_handlers::WindowHandlers;

pub struct Surfaces {
    pub surface: Surface,
    pub surface_khr: SurfaceKHR,
}

impl Surfaces {
    pub fn new(
        instance: &Instance,
        entry: &Entry,
        window: &Window,
    ) -> Self {
        let surface = Surface::new(entry, instance);
        let surface_khr = unsafe { ash_window::create_surface(entry, instance, window, None).unwrap() };

        info!("surface: {:?}", surface_khr);

        Self {
            surface,
            surface_khr,
        }
    }

    pub fn check_swapchain_support(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> bool {
        let swapchain_name = [Swapchain::name()];

        let extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .unwrap()
        };

        for required in swapchain_name.iter() {
            let found = extensions.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }

        true
    }

    pub fn require_surface_extension_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), Win32Surface::name().as_ptr()]
    }
}