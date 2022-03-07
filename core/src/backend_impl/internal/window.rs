use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::dpi::{PhysicalSize, Size};
use winit::monitor::MonitorHandle;
use winit::window::{WindowBuilder};
use winit::window::Fullscreen;

use crate::Result;
use crate::video::Display;
use crate::window::{Window, WindowConfig, WindowMode};

pub(crate) struct WindowImpl {
    event_loop: EventLoop<()>,
    window: winit::window::Window,
}

pub fn create_window<D: Into<Option<Display>>>(title: &str, display: D, config: &WindowConfig) -> Result<Window> {
    let event_loop = EventLoop::new();

    let display = match display.into() {
        Some(display) => Some(display),
        None => event_loop
            .primary_monitor()
            .map(|handle| handle.into()),
    };

    let mut window_builder = WindowBuilder::new()
        .with_title(title);

    let fullscreen = match config.mode {
        WindowMode::Windowed { size } => {
            let size = Size::Physical(size.into());

            window_builder = window_builder
                .with_inner_size(size)
                .with_resizable(true);

            None
        }
        WindowMode::Borderless => {
            Some(Fullscreen::Borderless(None))
        }
        /*
        WindowMode::Fullscreen {
            resolution,
            bit_depth,
            refresh_rate,
        } => {
            let video_mode = video_mode.clone().unwrap().into();

            Some(Fullscreen::Exclusive(video_mode))
        }
         */
    };

    let window_impl = {
        let window = window_builder
            .with_fullscreen(fullscreen)
            .build(&event_loop)?;

        WindowImpl {
            event_loop,
            window,
        }
    };

    Ok(window_impl.into())
}