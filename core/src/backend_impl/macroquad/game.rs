use hecs::World;
use macroquad::math::Rect;
use macroquad::prelude::scene::{self, Node, RefMut};
use macroquad::time::get_frame_time;

use crate::camera::{camera_position, Camera};
use crate::events::Event;
use crate::map::Map;
use crate::prelude::Transform;
use crate::state::GameState;
use crate::{storage, Result};

pub struct Game {
    event_queue: Vec<Event<()>>,
    states: HashMap<String, Box<dyn GameState>>,
    current_state_id: String,
}

impl Game {
    pub fn new(state_id: &str) -> Result<Self> {
        Ok(Game {
            event_queue: Vec::new(),
            states: HashMap::new(),
            current_state_id: state_id.to_string(),
        })
    }

    fn with_state<S: 'static + GameState>(self, state: S) -> Self {
        let mut states = self.states;
        states.insert(state.id(), state);

        Game { states, ..self }
    }

    fn current_state(&mut self) -> &mut Box<dyn GameState> {
        self.states.get()
    }

    pub fn set_state(&mut self, state_id: &str) -> Result<()> {
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
        node.state.draw().unwrap();
    }
}
