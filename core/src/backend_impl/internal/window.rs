use winit::event_loop::EventLoopWindowTarget;
use winit::dpi::{PhysicalSize, Size};
use winit::monitor::MonitorHandle;
use winit::window::WindowBuilder;
use winit::window::Fullscreen;

pub use winit::window::Window;

pub fn create_window<M: Into<Option<MonitorHandle>>>(window_target: &EventLoopWindowTarget<()>, monitor_handle: M, title: &str, config: &WindowConfig) -> Result<crate::context::Context> {
    let monitor_handle = match monitor_handle.into() {
        Some(monitor_handle) => Some(monitor_handle),
        None => window_target.primary_monitor(),
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
            Some(Fullscreen::Borderless(monitor_handle))
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
        .build(window_target)?;

    Ok(window)
}