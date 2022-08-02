use std::env;
use std::fs::File;
use std::io::Write;
use std::mem::swap;
use std::path::Path;
use ash::vk::{AccessFlags, CommandBufferBeginInfo, CommandBufferUsageFlags, DependencyFlags, Extent2D, Extent3D, Fence, Format, Image, ImageAspectFlags, ImageCopy, ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresource, ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, PipelineStageFlags, Queue, SampleCountFlags, SharingMode, SubmitInfo, WHOLE_SIZE};
use log::debug;
use log::Level::Debug;

use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use cotton::get_memory_type_index;
use cotton::renderer::acceleration_structures::AccelerationStructures;
use cotton::renderer::backends::Backends;
use cotton::renderer::images::Images;
use cotton::renderer::pipelines::Pipelines;

use cotton::renderer::render_passes::RenderPasses;
use cotton::renderer::Renderer;
use cotton::renderer::shader_module::ShaderModules;
use cotton::renderer::swapchains::Swapchains;
use cotton::scene::Scene;
use cotton::window_handlers::WindowHandlers;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    debug!("Start");

    //to_window();
    to_image();
}

fn to_window() {
    let window_size = winit::dpi::LogicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

    let window_handlers = WindowHandlers::new(window_size);

    let backends =
        cotton::renderer::backends::Backends::new(Some(&window_handlers), true)
            .expect("Failed to create backends");

    //backends.display_support_extension();

    let graphics_queue = backends.create_graphics_queue(0);
    let present_queue = backends.create_present_queue(0);

    let swapchains = Swapchains::new(&backends, window_size);
    let swapchain_images = swapchains.get_swapchain_images(&backends);

    let temp_image = Images::new(
        &backends,
        1,
        swapchains.format,
        Extent3D::builder()
            .width(swapchains.extent.width)
            .height(swapchains.extent.height)
            .depth(1)
            .build(),
        graphics_queue,
    );

    let render_passes = RenderPasses::new(&backends, swapchains.format, swapchain_images.image_views.clone(), swapchains.extent);

    let code = include_bytes!(env!("classical_raytracer_shader.spv"));
    let shader_modules = ShaderModules::new(&backends.device, code);

    let acceleration_structures = AccelerationStructures::new(
        &backends
    );

    let triangle_blas = acceleration_structures.create_triangle_blas(
        graphics_queue
    );

    let scene = Scene::build_scene(
        &backends,
        triangle_blas.get_device_address_info()
    );

    let tlas = acceleration_structures.create_tlas(
        scene,
        graphics_queue
    );

    let pipelines = Pipelines::new(
        &backends,
        shader_modules,
        swapchains.extent,
        &render_passes,
        &triangle_blas.mesh_buffer,
        tlas,
        graphics_queue,
        temp_image.image_views[0]
    );

    debug!("window close");
}

//TODO
fn to_image() {

    let extent3d = Extent3D::builder()
        .width(DEFAULT_WINDOW_WIDTH)
        .height(DEFAULT_WINDOW_HEIGHT)
        .depth(1)
        .build();
    let extent2d = Extent2D::builder()
        .width(extent3d.width)
        .height(extent3d.height)
        .build();

    let format = Format::R32G32B32A32_SFLOAT;

    let backends =
        cotton::renderer::backends::Backends::new(None, true)
            .expect("Failed to create backends");

    //backends.display_support_extension();

    let graphics_queue = backends.create_graphics_queue(0);

    let target_images = Images::new(&backends, 1, format, extent3d, graphics_queue);

    let render_passes = RenderPasses::new(
        &backends,
        format,
        target_images.image_views.clone(),
        extent2d,
    );

    let code = include_bytes!(env!("classical_raytracer_shader.spv"));
    let shader_modules = ShaderModules::new(&backends.device, code);

    let acceleration_structures = AccelerationStructures::new(
        &backends
    );

    let triangle_blas = acceleration_structures.create_triangle_blas(
        graphics_queue
    );

    let scene = Scene::build_scene(
        &backends,
        triangle_blas.get_device_address_info()
    );

    let tlas = acceleration_structures.create_tlas(
        scene,
        graphics_queue
    );

    let image = target_images.images[0];
    let image_view = target_images.image_views[0];

    let pipelines = Pipelines::new(
        &backends,
        shader_modules,
        extent2d,
        &render_passes,
        &triangle_blas.mesh_buffer,
        tlas,
        graphics_queue,
        image_view,
    );

    let renderer = Renderer::new(
        &backends,
        pipelines,
    );

    renderer.rendering(
        image,
        graphics_queue
    ).unwrap();

    save_image(
        &backends,
        image,
        format,
        extent3d,
        graphics_queue,
        "./out.png"
    ).unwrap();

    debug!("done");
}

fn save_image<P: AsRef<Path>>(
    backends: &Backends,
    target_image: Image,
    format: Format,
    extent3d: Extent3D,
    graphics_queue: Queue,
    image_file_path: P,
) -> anyhow::Result<()> {

    //transfer gpu to cpu

    let host_image_create_info = ImageCreateInfo::builder()
        .image_type(ImageType::TYPE_2D)
        .format(format)
        .extent(extent3d)
        .mip_levels(1)
        .initial_layout(ImageLayout::UNDEFINED)
        .array_layers(1)
        .samples(SampleCountFlags::TYPE_1)
        .tiling(ImageTiling::LINEAR)
        .usage(ImageUsageFlags::TRANSFER_DST)
        .sharing_mode(SharingMode::EXCLUSIVE)
        .build();

    let host_image = unsafe {
        backends.device.create_image(&host_image_create_info, None).unwrap()
    };

    let host_memory_requirement = unsafe {
        backends.device.get_image_memory_requirements(host_image)
    };
    let host_memory_alloc_info = MemoryAllocateInfo::builder()
        .allocation_size(host_memory_requirement.size)
        .memory_type_index(get_memory_type_index(
            &backends.device_memory_properties,
            host_memory_requirement.memory_type_bits,
            MemoryPropertyFlags::HOST_VISIBLE
                | MemoryPropertyFlags::HOST_COHERENT,
        ).unwrap()
        );

    let host_device_memory = unsafe {
        backends.device.allocate_memory(&host_memory_alloc_info, None).unwrap()
    };

    unsafe {
        backends.device.bind_image_memory(host_image, host_device_memory, 0).unwrap()
    }

    let command_pool = backends.create_graphics_command_pool();
    let command_buffers = backends.create_command_buffers(command_pool, 1);
    let command_buffer = command_buffers[0];

    unsafe {
        backends
            .device
            .begin_command_buffer(
                command_buffer,
                &CommandBufferBeginInfo::builder()
                    .flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                    .build(),
            ).unwrap();

        let image_barrier = ImageMemoryBarrier::builder()
            .src_access_mask(AccessFlags::empty())
            .dst_access_mask(AccessFlags::TRANSFER_WRITE)
            .old_layout(ImageLayout::UNDEFINED)
            .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
            .image(host_image)
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

        backends.device.cmd_pipeline_barrier(
            command_buffer,
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &[],
            &[],
            &[image_barrier],
        );

        //copy

        let copy_region = ImageCopy::builder()
            .src_subresource(
                ImageSubresourceLayers::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .build()
            )
            .dst_subresource(
                ImageSubresourceLayers::builder()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .build()
            )
            .extent(extent3d)
            .build();

        backends.device.cmd_copy_image(
            command_buffer,
            target_image,
            ImageLayout::GENERAL,
            host_image,
            ImageLayout::TRANSFER_DST_OPTIMAL,
            &[copy_region],
        );

        let image_barrier = ImageMemoryBarrier::builder()
            .src_access_mask(AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(AccessFlags::MEMORY_READ)
            .old_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(ImageLayout::GENERAL)
            .image(host_image)
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

        backends.device.cmd_pipeline_barrier(
            command_buffer,
            PipelineStageFlags::TRANSFER,
            PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &[],
            &[],
            &[image_barrier]
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
    }

    //png write

    let subresource = ImageSubresource::builder()
        .aspect_mask(ImageAspectFlags::COLOR)
        .build();

    let subresource_layout = unsafe {
        backends.device.get_image_subresource_layout(host_image, subresource)
    };

    let data: *const u8 = unsafe {
        backends
            .device
            .map_memory(
                host_device_memory,
                0,
                WHOLE_SIZE,
                MemoryMapFlags::empty(),
            ).unwrap() as _
    };

    let mut data = unsafe {
        data.offset(subresource_layout.offset as isize)
    };

    let mut png_encoder = png::Encoder::new(
        File::create(image_file_path).unwrap(),
        extent3d.width,
        extent3d.height,
    );

    png_encoder.set_depth(png::BitDepth::Eight);
    png_encoder.set_color(png::ColorType::Rgba);

    let mut png_writer = png_encoder
        .write_header()
        .unwrap()
        .into_stream_writer_with_size((4 * extent3d.width) as usize)
        .unwrap();

    //画像に詰めている時点で入っている値を全体の数で割って正規化していなかったのでここでしている?
    for _ in 0..extent3d.height {
        let row = unsafe {
            std::slice::from_raw_parts(data, 4 * 4 * extent3d.width as usize)
        };
        let row_f32: &[f32] = bytemuck::cast_slice(row);
        let row_rgba8: Vec<u8> = row_f32
            .iter()
            .map(|f| (256.0 * f.sqrt().clamp(0.0, 0.999)) as u8)
            .collect();

        png_writer.write_all(&row_rgba8).unwrap();
        data = unsafe {
            data.offset(subresource_layout.row_pitch as isize)
        };
    }

    png_writer.finish().unwrap();

    unsafe {
        backends.device.unmap_memory(host_device_memory);
        backends.device.free_memory(host_device_memory, None);
        backends.device.destroy_image(host_image, None);

        backends.device.free_command_buffers(command_pool, &command_buffers);
        backends.device.destroy_command_pool(command_pool, None);
    }

    debug!("save image");
    Ok(())
}