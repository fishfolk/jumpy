use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::dpi::{PhysicalSize, Size};
use winit::monitor::MonitorHandle;
use winit::window::{WindowBuilder};
use winit::window::Fullscreen;

use crate::Result;
use crate::video::Display;
use crate::window::{WindowConfig, WindowMode};

pub struct Window {
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

    let window = window_builder
        .with_fullscreen(fullscreen)
        .build(&event_loop)?;

    Ok(Window {
        event_loop,
        window,
    })
}

#[derive(Clone)]
pub struct Icon {
    pub small: [u8; 16 * 16 * 4],
    pub medium: [u8; 32 * 32 * 4],
    pub big: [u8; 64 * 64 * 4],
}