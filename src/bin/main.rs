use std::env;
use log::debug;

use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use cotton::renderer::acceleration_structures::AccelerationStructures;
use cotton::renderer::pipelines::Pipelines;

use cotton::renderer::render_passes::RenderPasses;
use cotton::renderer::shader_module::create_shader_module;
use cotton::renderer::swapchains::Swapchains;
use cotton::scene::Scene;
use cotton::window_handlers::WindowHandlers;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    to_window();
    //to_image();
}

fn to_window() {
    let window_size = winit::dpi::LogicalSize::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT);

    let window_handlers = WindowHandlers::new(window_size);

    let backends =
        cotton::renderer::backends::Backends::new(&window_handlers, true)
            .expect("Failed to create backends");

    //backends.display_support_extension();

    let graphics_queue = backends.create_graphics_queue(0);
    let present_queue = backends.create_present_queue(0);

    let swapchains = Swapchains::new(&backends, window_size);
    let swapchain_images = swapchains.get_swapchain_images(&backends.device);

    let render_passes = RenderPasses::new(&backends, swapchains.format, swapchain_images.image_views.clone(), swapchains.extent);

    let code = include_bytes!(env!("classical_raytracer_shader.spv"));
    let shader_module = create_shader_module(&backends.device, code);

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
        shader_module,
        swapchains.extent,
        &render_passes,
        graphics_queue
    );
}

//TODO
fn to_image() {

}