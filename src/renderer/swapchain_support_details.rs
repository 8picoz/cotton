use std::borrow::Borrow;
use std::ffi::CStr;
use ash::extensions::khr::Swapchain;
use ash::{Instance, vk};
use ash::vk::{Extent2D, PhysicalDevice, PresentInfoKHR, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR};
use log::info;
use winit::dpi::Size;
use crate::renderer::backends::surfaces::Surfaces;

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

    ///使用するformatsを選択する
    pub fn choose_swapchain_surface_format(&self) -> SurfaceFormatKHR {
        for available_format in self.formats.iter() {
            if available_format.format == vk::Format::R8G8B8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        *self.formats.first().unwrap()
    }

    pub fn choose_swapchain_present_mode(&self) -> PresentModeKHR {
        for available_present_mode in self.present_modes.iter() {
            let available_present_mode = available_present_mode.clone();

            //https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPresentModeKHR.html
            //MAILBOXは垂直同期
            if available_present_mode == PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        PresentModeKHR::FIFO
    }

    pub fn choose_swapchain_extent<S: Into<Size>>(&self, window_size: S) -> Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            return self.capabilities.current_extent;
        }

        let size = window_size.into();

        let width: u32 = size.to_logical(1.0).width;
        let height: u32 = size.to_logical(1.0).height;

        info!("swapchain size: width {}, height {}", width, height);

        let min = self.capabilities.min_image_extent;
        let max = self.capabilities.max_image_extent;
        let width = width.min(max.width).max(min.width);
        let height = height.min(max.height).max(min.height);

        Extent2D { width, height }
    }
}
