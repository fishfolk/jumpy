#[cfg(feature = "macroquad")]
#[path = "gui_impl/macroquad.rs"]
mod gui_impl;

#[cfg(not(feature = "macroquad"))]
#[path = "gui_impl/internal.rs"]
mod gui_impl;

pub use ff_core::gui::*;

pub use gui_impl::*;
