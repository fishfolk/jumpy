use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use winit::event_loop::EventLoop;

use winit::window::{Fullscreen, WindowBuilder};
use winit::window::{Window as WinitWindow, WindowId};

use crate::event::Event;
use crate::math::Size;
use crate::video::Display;
use crate::window::{WindowConfig, WindowMode};
use crate::Result;

static mut WINDOWS: Option<HashMap<WindowId, WinitWindow>> = None;

fn windows() -> &'static mut HashMap<WindowId, WinitWindow> {
    unsafe { WINDOWS.get_or_insert_with(HashMap::new) }
}

static mut PRIMARY_WINDOW: Option<WindowId> = None;

pub fn primary_window() -> Window {
    unsafe {
        PRIMARY_WINDOW
            .unwrap_or_else(|| {
                panic!("Attempted to get primary window but none exist! Have you created a window?")
            })
            .into()
    }
}

pub fn set_primary_window<W: Into<Window>>(window: W) {
    unsafe { PRIMARY_WINDOW = Some(window.into().id()) }
}

fn has_primary_window() -> bool {
    unsafe { PRIMARY_WINDOW.is_some() }
}

#[derive(Copy, Clone)]
pub struct Window(WindowId);

impl From<WindowId> for Window {
    fn from(id: WindowId) -> Self {
        Window(id)
    }
}

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

impl PartialEq<WindowId> for Window {
    fn eq(&self, other: &WindowId) -> bool {
        self.0 == other
    }
}

impl PartialEq<Window> for Window {
    fn eq(&self, other: &Window) -> bool {
        self.0 == other.0
    }
}

impl Eq for Window {}

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

    let id = window.id();

    windows().insert(id, window);

    if !has_primary_window() {
        set_primary_window(id);
    }

    Ok(Window(id))
}

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

pub struct WindowIcon {}
