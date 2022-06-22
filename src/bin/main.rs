use std::env;
use ash::extensions::khr::Swapchain;
use log::info;
use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDHT};
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
    let window_size = winit::dpi::LogicalSize::new(DEFAULT_WINDOW_WIDHT, DEFAULT_WINDOW_HEIGHT);

    let window_handlers = WindowHandlers::new(window_size);

    let backends =
        cotton::renderer::backends::Backends::new(&window_handlers, true)
            .expect("Failed to create backends");

    let graphics_queue = backends.create_graphics_queue(0);
    let present_queue = backends.create_present_queue(0);

    let swapchains = Swapchains::new(&backends, window_size.clone());

    let swapchain_images = swapchains.get_swapchain_images();
}

//TODO
fn to_image() {

}