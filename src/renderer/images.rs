use std::ops::Deref;
use ash::Device;
use ash::vk::{AccessFlags, CommandBufferBeginInfo, CommandBufferUsageFlags, ComponentMapping, ComponentSwizzle, DependencyFlags, Extent2D, Extent3D, Fence, Format, Image, ImageAspectFlags, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, MemoryAllocateInfo, MemoryPropertyFlags, PipelineStageFlags, Queue, SampleCountFlags, SharingMode, SubmitInfo};
use log::debug;
use crate::get_memory_type_index;
use crate::renderer::backends::Backends;

pub struct Images<'a> {
    backends: &'a Backends,
    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,
}

impl<'a> Images<'a> {
    pub fn new(
        backends: &'a Backends,
        count: usize,
        format: Format,
        extent: Extent3D,
        graphics_queue: Queue,
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
            backends.device.create_image(&image_create_info, None).unwrap()
        };

        let memory_requirement = unsafe {
            backends.device.get_image_memory_requirements(image)
        };

        let memory_alloc_info = MemoryAllocateInfo::builder()
            .allocation_size(memory_requirement.size)
            .memory_type_index(
                get_memory_type_index(
                    &backends.device_memory_properties,
                    memory_requirement.memory_type_bits,
                    MemoryPropertyFlags::DEVICE_LOCAL,
                ).unwrap()
            );

        let device_memory = unsafe {
            backends.device.allocate_memory(&memory_alloc_info, None).unwrap()
        };

        unsafe {
            backends.device.bind_image_memory(image, device_memory, 0).unwrap();
        }
        
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
            backends.device.create_image_view(&image_view_create_info, None).unwrap()
        };

        //Initialize
        let command_pool = backends.create_graphics_command_pool();
        let command_buffers = backends.create_command_buffers(command_pool, 1);
        let command_buffer = command_buffers[0];

        unsafe {
            backends.device.begin_command_buffer(
                command_buffer,
                &CommandBufferBeginInfo::builder()
                    .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                    .build(),
            ).unwrap();

            let image_barrier = ImageMemoryBarrier::builder()
                .src_access_mask(AccessFlags::empty())
                .dst_access_mask(AccessFlags::empty())
                .old_layout(ImageLayout::UNDEFINED)
                .new_layout(ImageLayout::GENERAL)
                .image(image)
                .subresource_range(
                    ImageSubresourceRange::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .level_count(1)
                        .base_array_layer(1)
                        .build()
                ).build();

            backends.device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::ALL_COMMANDS,
                PipelineStageFlags::ALL_COMMANDS,
                DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );

            backends.device.end_command_buffer(command_buffer).unwrap();
        }

        let submit_infos = [
            SubmitInfo::builder()
                .command_buffers(&command_buffers)
                .build()
        ];

        unsafe {
            backends
                .device
                .queue_submit(graphics_queue, &submit_infos, Fence::null())
                .expect("queue submit failed");

            backends.device.queue_wait_idle(graphics_queue).unwrap();
            backends.device.free_command_buffers(command_pool, &command_buffers);
            backends.device.destroy_command_pool(command_pool, None);
        }

        debug!("Image: {:?}, ImageView: {:?}", image, image_view);

        Self {
            backends,
            //一つだけ生成
            images: vec![image],
            image_views: vec![image_view],
        }
    }

    //画像単体で出力したいならvk::Imageを素のまま作ってそこに保存すれば良い
    pub fn create_images_for_swapchain_images(
        backends: &'a Backends,
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

            image_views.push(unsafe { backends.device.create_image_view(&create_info, None) }.unwrap());
        }

        debug!("Create Swapchain Image Views");

        Self {
            backends,
            images,
            image_views,
        }
    }
}

impl Drop for Images<'_> {
    fn drop(&mut self) {
        unsafe {
            for image_view in self.image_views.clone() {
                self.backends.device.destroy_image_view(image_view, None);
            }
        }
    }
}