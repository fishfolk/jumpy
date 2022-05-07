use crate::config::Config;
use crate::event::Event;
use crate::gl::init_gl_context;
use crate::input::init_gamepad_context;
use crate::prelude::vertex::VertexImpl;
use crate::rendering::renderer::init_renderer;
use crate::rendering::Vertex;
use crate::result::Result;
use crate::window::create_window;
use glutin::event_loop::EventLoop;
use std::fmt::Debug;

pub async fn init_context<E: 'static + Debug>(
    window_title: &str,
    event_loop: &EventLoop<Event<E>>,
    config: &Config,
) -> Result<()> {
    let window = create_window(window_title, event_loop, config)?;
    let _ = init_gl_context(window);
    init_renderer(&config.video)?;
    init_gamepad_context().await?;

    Ok(())
}
