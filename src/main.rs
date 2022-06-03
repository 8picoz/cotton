use std::env;

mod classical_raytracer;
mod window_handlers;
pub mod constants;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();


}