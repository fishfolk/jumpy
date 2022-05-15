use std::ops::Deref;

use egui_glow::Painter;

pub use egui::SidePanel;

use crate::gl::gl_context;
use crate::math::{vec2, AsVec2, Rect, Size, Vec2};
use crate::render::renderer::renderer;
use crate::window::{context_wrapper, window};

pub struct GuiContext {
    egui_glow: egui_glow::EguiGlow,
    should_redraw: bool,
}

impl GuiContext {
    pub fn new() -> Self {
        GuiContext {
            egui_glow: egui_glow::EguiGlow::new(context_wrapper().window(), gl_context()),
            should_redraw: false,
        }
    }

    pub fn egui_ctx(&mut self) -> &mut egui::Context {
        &mut self.egui_glow.egui_ctx
    }

    pub fn egui_winit(&mut self) -> &mut egui_glow::egui_winit::State {
        &mut self.egui_glow.egui_winit
    }

    pub fn gl_painter(&mut self) -> &mut egui_glow::Painter {
        &mut self.egui_glow.painter
    }

    pub fn handle(&mut self, event: &glutin::event::WindowEvent<'_>) -> bool {
        self.egui_glow.on_event(event)
    }

    pub fn build(&mut self, f: impl FnMut(&egui::Context)) {
        #[cfg(debug_assertions)]
        if !self.should_redraw {
            println!(
                "WARNING: Attempting to build an UI when the previous one has not yet been drawn!"
            )
        }

        let res = self.egui_glow.run(window(), f);
        self.should_redraw = self.should_redraw | res;
    }

    pub fn draw(&mut self) {
        if self.should_redraw {
            renderer().draw_batch();
            self.egui_glow.paint(window());
        }
    }
}

impl Default for GuiContext {
    fn default() -> Self {
        Self::new()
    }
}

static mut GUI_CONTEXT: Option<GuiContext> = None;

pub fn gui_context() -> &'static mut GuiContext {
    unsafe {
        GUI_CONTEXT.as_mut().unwrap_or_else(|| {
            panic!("ERROR: Attempted to retrieve gui context but it does not exist!!")
        })
    }
}

pub fn create_gui_context() {
    unsafe { GUI_CONTEXT = Some(GuiContext::default()) };
}

pub fn destroy_gui_context() {
    let mut ctx = unsafe { GUI_CONTEXT.take() }
        .unwrap_or_else(|| panic!("ERROR: Attempted to destroy gui context but none exists!"));
    ctx.egui_glow.destroy();
}

pub fn build_gui(f: impl FnMut(&egui::Context)) {
    gui_context().build(f);
}

pub fn draw_gui() {
    gui_context().draw()
}

pub trait ToEguiVec2 {
    fn to_egui_vec2(self) -> egui::Vec2;
}

pub trait ToEguiPos2 {
    fn to_egui_pos2(self) -> egui::Pos2;
}

impl ToEguiVec2 for Vec2 {
    fn to_egui_vec2(self) -> egui::Vec2 {
        egui::Vec2::new(self.x, self.y)
    }
}

impl ToEguiPos2 for Vec2 {
    fn to_egui_pos2(self) -> egui::Pos2 {
        egui::Pos2::new(self.x, self.y)
    }
}

impl AsVec2 for egui::Vec2 {
    fn as_vec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

impl AsVec2 for egui::Pos2 {
    fn as_vec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

impl From<Size<f32>> for egui::Vec2 {
    fn from(size: Size<f32>) -> Self {
        egui::Vec2::new(size.width, size.height)
    }
}

impl From<egui::Vec2> for Size<f32> {
    fn from(vec: egui::Vec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<Rect> for egui::Rect {
    fn from(rect: Rect) -> Self {
        let min = rect.point();
        let max = min + rect.size();
        egui::Rect::from_min_max(min.to_egui_pos2(), max.to_egui_pos2())
    }
}

impl From<egui::Rect> for Rect {
    fn from(rect: egui::Rect) -> Self {
        let pos = rect.min.as_vec2();
        let size: Size<f32> = (rect.max.as_vec2() - pos).into();
        (pos, size).into()
    }
}
