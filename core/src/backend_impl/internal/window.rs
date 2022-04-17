use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use winit::event_loop::EventLoop;

use winit::window::Window as WinitWindow;
use winit::window::{Fullscreen, WindowBuilder};

use crate::event::Event;
use crate::math::Size;
use crate::video::Display;
use crate::window::{WindowConfig, WindowMode};
use crate::Result;

static mut LAST_WINDOW_ID: usize = 0;

fn window_id() -> usize {
    unsafe {
        LAST_WINDOW_ID += 1;
        LAST_WINDOW_ID
    }
}

static mut WINDOWS: Option<HashMap<usize, WinitWindow>> = None;

fn windows() -> &'static mut HashMap<usize, WinitWindow> {
    unsafe { WINDOWS.get_or_insert_with(HashMap::new) }
}

static mut ACTIVE_WINDOW: Option<usize> = None;

pub fn is_active_window_set() -> bool {
    unsafe { ACTIVE_WINDOW.is_some() }
}

pub fn active_window() -> Window {
    let id = unsafe {
        ACTIVE_WINDOW
            .unwrap_or_else(|| panic!("Attempted to get active window but none has been set!"))
    };

    Window(id)
}

pub fn set_active_window(window: &Window) {
    unsafe { ACTIVE_WINDOW = Some(window.0) }
}

pub struct Window(usize);

impl Deref for Window {
    type Target = WinitWindow;

    fn deref(&self) -> &Self::Target {
        windows().get(&self.0).unwrap()
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        windows().get_mut(&self.0).unwrap()
    }
}

pub fn window_size() -> Size<u32> {
    let size = active_window().inner_size();

    Size {
        width: size.width,
        height: size.height,
    }
}

pub fn create_window<E: 'static + Debug>(
    title: &str,
    event_loop: &EventLoop<Event<E>>,
    config: &WindowConfig,
) -> Result<Window> {
    let mut window_builder = WindowBuilder::new().with_title(title);

    /*
    let _display = match display.into() {
        Some(display) => Some(display),
        None => event_loop.primary_monitor().map(|handle| handle.into()),
    };
    */

    window_builder = match config.mode {
        WindowMode::Windowed { size } => {
            let size = winit::dpi::Size::Physical(size.into());

            window_builder
                .with_fullscreen(None)
                .with_inner_size(size)
                .with_resizable(true)
        }
        WindowMode::Borderless => {
            let fullscreen = Fullscreen::Borderless(None);

            window_builder.with_fullscreen(Some(fullscreen))
        }
        WindowMode::Fullscreen {
            resolution,
            bit_depth,
            refresh_rate,
        } => {
            //let video_mode = video_mode.clone().unwrap().into();

            //let fullscreen = Fullscreen::Exclusive(video_mode);

            let fullscreen = Fullscreen::Borderless(None);

            window_builder.with_fullscreen(Some(fullscreen))
        }
    };

    let window = window_builder.build(event_loop)?;

    let id = window_id();

    windows().insert(id, window);

    Ok(Window(id))
}

pub struct WindowIcon {}

pub(crate) fn apply_window_config(config: &WindowConfig) {
    match config.mode {
        WindowMode::Windowed { size } => {
            let size = winit::dpi::Size::Physical(size.into());

            let window = active_window();

            window.set_fullscreen(None);
            window.set_inner_size(size);
            window.set_resizable(true);
        }
        WindowMode::Borderless => {
            let fullscreen = Fullscreen::Borderless(None);

            let window = active_window();

            window.set_fullscreen(Some(fullscreen));
            window.set_resizable(false);
        }
        WindowMode::Fullscreen {
            resolution,
            bit_depth,
            refresh_rate,
        } => {
            //let video_mode = video_mode.clone().unwrap().into();

            //let fullscreen = Fullscreen::Exclusive(video_mode);

            let fullscreen = Fullscreen::Borderless(None);

            let window = active_window();

            window.set_fullscreen(Some(fullscreen));
            window.set_resizable(false);
        }
    }
}
