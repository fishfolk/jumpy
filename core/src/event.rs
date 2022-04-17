pub use crate::backend_impl::event::*;

use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter, Write};

use crate::error::ErrorKind;
use crate::prelude::GameState;
use crate::state::GameStateBuilderFn;
use crate::{Config, Error, Result};

pub type DefaultCustomEvent = ();

/// This holds all the event types
pub enum Event<T: 'static + Debug> {
    /// Custom event
    Custom(T),
    /// Config changed
    ConfigChanged(Config),
    /// Change game state
    StateTransition(String, GameStateBuilderFn),
    /// Quit to desktop
    Quit,
}

impl<T: 'static + Debug> Debug for Event<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Custom(event) => format!("Event::Custom({:?})", &event).fmt(f),
            Event::ConfigChanged(..) => format!("Event::ConfigChanged(Config)").fmt(f),
            Event::StateTransition(id, ..) => format!("Event::StateTransition({}, fn)", id).fmt(f),
            Event::Quit => format!("Event::Quit").fmt(f),
        }
    }
}
