use ash::vk::{AccessFlags, ClearColorValue, CommandBufferBeginInfo, CommandBufferResetFlags, CommandBufferUsageFlags, DependencyFlags, Fence, Image, ImageAspectFlags, ImageLayout, ImageMemoryBarrier, ImageSubresourceRange, ImageView, PipelineStageFlags, Queue, SubmitInfo};
use log::debug;
use crate::renderer::backends::Backends;
use crate::renderer::pipelines::Pipelines;

pub mod backends;
pub mod swapchains;
pub mod images;
pub mod validation_layer;
pub mod swapchain_support_details;
pub mod render_passes;
pub mod pipelines;
pub mod acceleration_structures;
pub mod mesh_buffer;
pub mod shader_module;

pub struct Renderer<'a> {
    backends: &'a Backends,
    pipelines: Pipelines<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        backends: &'a Backends,
        pipelines: Pipelines<'a>
    ) -> Self {
        Self {
            backends,
            pipelines,
        }
    }

    pub fn rendering(
        &self,
        image: Image,
        graphics_queue: Queue,
    ) -> anyhow::Result<()> {
        debug!("rendering");

        let command_buffer_begin_info = CommandBufferBeginInfo::builder()
            .flags(CommandBufferUsageFlags::SIMULTANEOUS_USE)
            .build();

        let command_pool = self.backends.create_graphics_command_pool();
        let command_buffers = self.backends.create_command_buffers(command_pool, 1);
        let command_buffer = command_buffers[0];

        //clear image
        unsafe {
            self.backends
                .device
                .reset_command_buffer(command_buffer, CommandBufferResetFlags::RELEASE_RESOURCES)
                .unwrap();

            self
                .backends
                .device
                .begin_command_buffer(
                    command_buffer,
                    &command_buffer_begin_info
                ).unwrap();

            let range = ImageSubresourceRange::builder()
                .aspect_mask(ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build();

            self.backends.device.cmd_clear_color_image(
                command_buffer,
                image,
                ImageLayout::GENERAL,
                &ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0],
                },
                &[range]
            );

            let image_barrier = ImageMemoryBarrier::builder()
                .src_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(AccessFlags::SHADER_WRITE | AccessFlags::SHADER_READ)
                .old_layout(ImageLayout::GENERAL)
                .new_layout(ImageLayout::GENERAL)
                .image(image)
                .subresource_range(
                    ImageSubresourceRange::builder()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1)
                        .build(),
                )
                .build();

            self.backends.device.cmd_pipeline_barrier(
                command_buffer,
                PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                PipelineStageFlags::RAY_TRACING_SHADER_KHR,
                DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier]
            );

            self.backends.device.end_command_buffer(command_buffer).unwrap();
        }

        let submit_infos = [
            SubmitInfo::builder()
                .command_buffers(&[command_buffer])
                .build()
        ];

        unsafe {
            self.backends
                .device
                .queue_submit(graphics_queue, &submit_infos, Fence::null())
                .unwrap();

            self.backends
                .device
                .queue_wait_idle(graphics_queue)
                .unwrap();
            self.backends
                .device
                .free_command_buffers(
                    command_pool,
                    &[command_buffer]
                );
        }

        Ok(())
    }
}