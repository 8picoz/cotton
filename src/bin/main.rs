use std::env;



use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};
use cotton::renderer::pipelines::Pipelines;

use cotton::renderer::render_passes::RenderPasses;
use cotton::renderer::swapchains::Swapchains;
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

    backends.display_support_extension();

    let graphics_queue = backends.create_graphics_queue(0);
    let present_queue = backends.create_present_queue(0);

    let swapchains = Swapchains::new(&backends, window_size);
    let swapchain_images = swapchains.get_swapchain_images(&backends.device);

    let render_passes = RenderPasses::new(&backends, swapchains.format, swapchain_images.image_views.clone(), swapchains.extent);

    let pipelines = Pipelines::new(
        &backends.device,
        
    );
}

//TODO
fn to_image() {

}