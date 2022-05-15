use std::fmt::Debug;

use glutin::event_loop::EventLoop;
use glutin::window::Window;
use glutin::{ContextWrapper, PossiblyCurrent};

use crate::config::Config;
use crate::event::Event;
use crate::gl::init_gl_context;
use crate::gui::{create_gui_context, destroy_gui_context, GuiContext};
use crate::input::init_gamepad_context;
use crate::prelude::vertex::VertexImpl;
use crate::prelude::{create_audio_context, destroy_audio_context};
use crate::render::renderer::{create_renderer, destroy_renderer, Renderer};
use crate::render::Vertex;
use crate::result::Result;
use crate::texture::destroy_textures;
use crate::window::create_window;

pub async fn create_context<E: 'static + Debug>(
    window_title: &str,
    event_loop: &EventLoop<Event<E>>,
    config: &Config,
) -> Result<()> {
    create_audio_context();
    let window = create_window(window_title, event_loop, config)?;
    let _ = init_gl_context(window);
    create_gui_context();
    create_renderer(&config.video)?;
    init_gamepad_context().await?;

    Ok(())
}

pub fn destroy_context() {
    destroy_audio_context();
    destroy_gui_context();
    destroy_renderer();
    destroy_textures();
}
