use std::borrow::Borrow;
use std::path::is_separator;
use ash::extensions::khr::Swapchain;
use ash::vk;
use ash::vk::{CompositeAlphaFlagsKHR, Extent2D, Format, Image, ImageUsageFlags, SharingMode, SwapchainCreateInfoKHR, SwapchainKHR};
use log::{debug, info};
use winit::dpi::{LogicalSize, Size};
use crate::renderer::backends::Backends;
use crate::renderer::queue_family_indices::QueueFamilyIndices;
use crate::renderer::surfaces::Surfaces;
use crate::renderer::swapchain_support_details::SwapchainSupportDetails;

pub struct Swapchains {
    pub swapchain: Swapchain,
    pub swapchain_khr: SwapchainKHR,
    pub format: Format,
    pub extent: Extent2D,
}

impl Swapchains {
    pub fn new<S: Into<Size>>(
        backends: &Backends,
        window_size: S,
    ) -> Self {
        //as_refはOptionなどの中身に対してborrowすることが出来る
        let surfaces = backends.surfaces.as_ref().expect("Not found surfaces");

        let swapchain_support = SwapchainSupportDetails::new(backends.physical_device, surfaces);

        let surface_format = swapchain_support.choose_swapchain_surface_format();
        let present_mode = swapchain_support.choose_swapchain_present_mode();
        let extent = swapchain_support.choose_swapchain_extent(window_size);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;

        if swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut create_info = SwapchainCreateInfoKHR::builder()
            .surface(surfaces.surface_khr)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT);

        let indices = QueueFamilyIndices::new(
            &backends.instance,
            backends.surfaces.as_ref(),
            backends.physical_device
        );

        let graphics_family = indices.graphics_family.expect("Graphics family is not exist");
        let present_family = indices.present_family.expect("Present family is not exist");

        let queue_family_indices = [
            graphics_family,
            present_family,
        ];

        info!("Graphics family: {}, Present family: {}", graphics_family, present_family);

        create_info = if graphics_family != present_family {
            create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices)
        } else {
            create_info.image_sharing_mode(SharingMode::EXCLUSIVE)
        };

        let create_info = create_info
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(SwapchainKHR::null())
            .build();

        let swapchain = Swapchain::new(&backends.instance, &backends.device);
        let swapchain_khr = unsafe { swapchain.create_swapchain(&create_info, None).unwrap() };

        info!("swapchain: {:?}", swapchain_khr);

        Self {
            swapchain,
            swapchain_khr,
            format: surface_format.format,
            extent,
        }
    }
    //画像単体で出力したいならvk::Imageを素のまま作ってそこに保存すれば良い
    pub fn get_swapchain_images(&self) -> Vec<Image> {
        unsafe { self.swapchain.get_swapchain_images(self.swapchain_khr).unwrap() }
    }
}

impl Drop for Swapchains {
    fn drop(&mut self) {
        unsafe {
            self.swapchain.destroy_swapchain(self.swapchain_khr, None);
        }
    }
}
