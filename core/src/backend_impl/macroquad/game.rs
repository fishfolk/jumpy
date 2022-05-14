use std::collections::HashMap;
use std::time::Duration;

use hecs::World;
use macroquad::math::Rect;
use macroquad::prelude::scene::{self, Node, RefMut};
use macroquad::time::get_frame_time;

use crate::camera::{camera_position, Camera};
use crate::event::Event;
use crate::map::Map;
use crate::physics::fixed_delta_time;
use crate::prelude::Transform;
use crate::result::Result;
use crate::state::{GameState, GameStateBuilderFn};
use crate::storage;
use crate::viewport::{resize_viewport, viewport_size};
use crate::window::window_size;

pub fn delta_time() -> Duration {
    Duration::from_secs_f32(get_frame_time())
}

pub struct Game {
    event_queue: Vec<Event<()>>,
    state: Box<dyn GameState>,
}

impl Game {
    pub fn new<S: 'static + GameState>(state: S) -> Result<Self> {
        Ok(Game {
            event_queue: Vec::new(),
            state: Box::new(state),
        })
    }

    pub fn change_state(&mut self, state: Box<dyn GameState>) -> Result<()> {
        let world = self.state.end()?;

        self.state = state;

        self.state.begin(world.into())?;

        Ok(())
    }
}

impl Node for Game {
    fn ready(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.state.begin(None).unwrap();
    }

    fn update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.state.update(get_frame_time()).unwrap();
    }

    fn fixed_update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        node.state.fixed_update(get_frame_time(), 1.0).unwrap();
    }

    fn draw(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        let window_size = window_size();
        let viewport_size = viewport_size();
        if viewport_size != window_size {
            resize_viewport(window_size.width, window_size.height);
        }

        node.state.draw(get_frame_time()).unwrap();
    }
}
