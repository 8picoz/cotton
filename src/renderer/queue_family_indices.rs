use ash::Instance;
use ash::vk::{PhysicalDevice, QueueFlags};
use crate::renderer::surfaces::Surfaces;
use crate::renderer::swapchain_support_details::SwapchainSupportDetails;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new(
        instance: &Instance,
        surfaces: Option<&Surfaces>,
        physical_device: PhysicalDevice,
    ) -> QueueFamilyIndices {
        let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics_family = None;
        let mut present_family = None;

        for (i, queue) in queue_families.iter().enumerate() {
            if queue.queue_flags.contains(QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }

            if let Some(surfaces) = surfaces {
                let present_support = unsafe {
                    surfaces.surface.get_physical_device_surface_support(physical_device, i as u32, surfaces.surface_khr)
                };

                if present_support.unwrap() {
                    present_family = Some(i as u32);
                }
            }
        }

        Self {
            graphics_family,
            present_family,
        }
    }

    //with surface
    pub fn is_device_suitable(
        &self,
        instance: &Instance,
        physical_device: PhysicalDevice,
        surfaces: &Surfaces,
    ) -> bool {
        let extension_support = Surfaces::check_swapchain_support(instance, physical_device);

        let mut swapchain_adequate = false;

        if extension_support {
            let swapchain_support_details = SwapchainSupportDetails::new(physical_device, surfaces);

            swapchain_adequate = !swapchain_support_details.formats.is_empty()
                && !swapchain_support_details.present_modes.is_empty();
        }

        self.is_all_complete() && extension_support && swapchain_adequate
    }

    //with surface
    fn is_all_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}