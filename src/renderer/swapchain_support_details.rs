use std::ffi::CStr;
use ash::extensions::khr::Swapchain;
use ash::{Instance, vk};
use ash::vk::{PhysicalDevice, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR};
use crate::renderer::surfaces::Surfaces;

pub struct SwapchainSupportDetails {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn new(
        physical_device: PhysicalDevice,
        surfaces: &Surfaces,
    ) -> Self {
        let capabilities = unsafe {
            surfaces.surface
                .get_physical_device_surface_capabilities(physical_device, surfaces.surface_khr)
                .unwrap()
        };

        let formats = unsafe {
            surfaces.surface
                .get_physical_device_surface_formats(physical_device, surfaces.surface_khr)
                .unwrap()
        };

        let present_modes = unsafe {
            surfaces.surface
                .get_physical_device_surface_present_modes(physical_device, surfaces.surface_khr)
                .unwrap()
        };

        Self {
            capabilities,
            formats,
            present_modes
        }
    }

    pub fn check_swapchain_support(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> bool {
        let required_extension = Swapchain::name();

        let extensions = unsafe {
            instance.enumerate_device_extension_properties(physical_device).unwrap()
        };


        let found = extensions.iter().any(|ext| {
            let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            required_extension == name
        });

        if !found {
            return false;
        }

        true
    }
}
