use winit::dpi::Size;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use crate::constants;

pub struct WindowHandlers {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

impl WindowHandlers {
    pub fn new<S: Into<Size>>(window_size: S) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("cotton")
            .with_inner_size(window_size)
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();

        Self {
            event_loop,
            window,
        }
    }
}