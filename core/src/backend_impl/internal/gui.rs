use crate::gl::gl_context;
use crate::window::{context_wrapper, window};
use egui_glow::Painter;
use std::ops::Deref;

static mut GUI_PAINTER: Option<egui_glow::EguiGlow> = None;

pub fn gui_context() -> &'static mut egui_glow::EguiGlow {
    unsafe {
        GUI_PAINTER.as_mut().unwrap_or_else(|| {
            panic!("ERROR: Attempted to retrieve gui painter but gui has not been initialized yet!")
        })
    }
}

pub fn init_gui() {
    let gui = egui_glow::EguiGlow::new(context_wrapper().window(), gl_context());
    unsafe { GUI_PAINTER = Some(gui) };
}

pub fn draw_gui() {
    gui_context().paint(window());
}

pub fn destroy_gui() {
    let mut gui_painter = unsafe { GUI_PAINTER.take() }
        .unwrap_or_else(|| panic!("ERROR: Attempted to destroy gui context but none exists!"));
    gui_painter.destroy();
}
