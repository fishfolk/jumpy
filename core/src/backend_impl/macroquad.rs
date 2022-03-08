#[path = "macroquad/text.rs"]
pub mod text;

#[path = "macroquad/window.rs"]
pub mod window;

#[path = "macroquad/texture.rs"]
pub mod texture;

#[path = "macroquad/input.rs"]
pub mod input;

#[path = "macroquad/file.rs"]
pub mod file;

#[path = "macroquad/color.rs"]
pub mod color;

#[path = "macroquad/rendering.rs"]
pub mod rendering;

#[path = "macroquad/viewport.rs"]
pub mod viewport;

pub mod video {}

pub use macroquad::math;

pub use macroquad::ui;

pub use macroquad::experimental::scene;

pub use macroquad::camera;