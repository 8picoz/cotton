use std::env;
use crate::window_handlers::WindowHandlers;

mod classical_raytracer;
mod window_handlers;
pub mod constants;
mod renderer;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    let window_handlers = WindowHandlers::new(constants::DEFAULT_WINDOW_WIDHT, constants::DEFAULT_WINDOW_HEIGHT);


}