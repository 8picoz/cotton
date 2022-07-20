use std::ops::Deref;
use ash::Device;
use ash::vk::{ComponentMapping, ComponentSwizzle, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateInfo, ImageViewType};
use log::debug;

pub struct Images<'a> {
    pub device: &'a Device,
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
}

impl<'a> Images<'a> {
    //画像単体で出力したいならvk::Imageを素のまま作ってそこに保存すれば良い
    pub fn new(
        device: &'a Device,
        images: Vec<Image>,
        swapchain_image_format: Format,
    ) -> Self {
        let mut image_views = vec![];

        for image in images.iter() {
            let create_info = ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .format(swapchain_image_format)
                .components(
                    ComponentMapping::builder()
                        .r(ComponentSwizzle::IDENTITY)
                        .g(ComponentSwizzle::IDENTITY)
                        .b(ComponentSwizzle::IDENTITY)
                        .a(ComponentSwizzle::IDENTITY)
                        .build(),
                )
                .subresource_range(
                    //画像自体の目的
                    ImageSubresourceRange::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_array_layer(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .build();

            image_views.push(unsafe { device.create_image_view(&create_info, None) }.unwrap());
        }

        debug!("Create Swapchain Image Views");

        Self {
            device,
            images,
            image_views,
        }
    }
}

impl Drop for Images<'_> {
    fn drop(&mut self) {
        unsafe {
            for image_view in self.image_views.clone() {
                self.device.destroy_image_view(image_view, None);
            }
        }
    }
}