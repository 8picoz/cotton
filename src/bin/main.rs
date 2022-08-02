use std::env;
use std::mem::swap;
use ash::vk::{Extent2D, Extent3D, Format};
use log::debug;
use log::Level::Debug;

use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use cotton::renderer::acceleration_structures::AccelerationStructures;
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

    let target_image = Images::new(&backends, 1, format, extent3d, graphics_queue);

    let render_passes = RenderPasses::new(
        &backends,
        format,
        target_image.image_views.clone(),
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

    let image = target_image.images[0];
    let image_view = target_image.image_views[0];

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

    save_image().unwrap();

    debug!("done");
}

fn save_image() -> anyhow::Result<()> {


    debug!("save image");
    Ok(())
}