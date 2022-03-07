use crate::math::vec2;

use crate::video::*;

impl From<winit::monitor::VideoMode> for VideoMode {
    fn from(mode: winit::monitor::VideoMode) -> Self {
        VideoMode {
            resolution: mode.size().into(),
            bit_depth: mode.bit_depth(),
            refresh_rate: mode.refresh_rate(),
            display: Some(mode.monitor().into()),
        }
    }
}

impl From<winit::monitor::MonitorHandle> for Display {
    fn from(handle: winit::monitor::MonitorHandle) -> Self {
        let position = handle.position();

        Display {
            name: handle.name(),
            resolution: handle.size().into(),
            position: vec2(position.x as f32, position.y as f32),
            scale_factor: handle.scale_factor() as f32,
            video_modes: handle
                .video_modes()
                .map(|mode| mode.clone().into())
                .collect(),
        }
    }
}