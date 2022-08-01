use std::ops::Deref;
use ash::Device;
use ash::vk::{ComponentMapping, ComponentSwizzle, Extent2D, Extent3D, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, SampleCountFlags, SharingMode};
use log::debug;
use crate::renderer::backends::Backends;

pub struct Images<'a> {
    pub device: &'a Device,
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
}

impl<'a> Images<'a> {
    pub fn new(
        device: &'a Device,
        count: usize,
        format: Format,
        extent: Extent3D,
    ) -> Self {

        let image_create_info = ImageCreateInfo::builder()
            .image_type(ImageType::TYPE_2D)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(SampleCountFlags::TYPE_1)
            .tiling(ImageTiling::OPTIMAL)
            .usage(
                ImageUsageFlags::COLOR_ATTACHMENT
                    | ImageUsageFlags::TRANSFER_DST
                    | ImageUsageFlags::STORAGE
                    | ImageUsageFlags::TRANSFER_SRC,
            )
            .sharing_mode(SharingMode::EXCLUSIVE)
            .build();

        let image = unsafe {
            device.create_image(&image_create_info, None).unwrap()
        };
        
        let image_view_create_info = ImageViewCreateInfo::builder()
            .view_type(ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(
                ImageSubresourceRange::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build()
            )
            .image(image)
            .build();

        let image_view = unsafe {
            device.create_image_view(&image_view_create_info, None).unwrap()
        };

        debug!("Image: {:?}, ImageView: {:?}", image, image_view);

        Self {
            device,
            //一つだけ生成
            images: vec![image],
            image_views: vec![image_view],
        }
    }

    //画像単体で出力したいならvk::Imageを素のまま作ってそこに保存すれば良い
    pub fn create_images_for_swapchain_images(
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