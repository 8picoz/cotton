use std::env;
use cotton::constants::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDHT};
use cotton::window_handlers::WindowHandlers;

fn main() {
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    to_window();
    //to_image();
}

fn to_window() {
    let window_handlers = WindowHandlers::new(DEFAULT_WINDOW_WIDHT, DEFAULT_WINDOW_HEIGHT);


}

//TODO
fn to_image() {

}