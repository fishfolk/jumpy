pub use crate::backend_impl::event::*;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Write};
use std::rc::Rc;

use crate::error::ErrorKind;
use crate::prelude::GameState;
use crate::state::GameStateBuilderFn;
use crate::{Config, Error, Result};

/// This holds all the event types
pub enum Event<T: 'static + Debug> {
    /// Custom event
    Custom(T),
    /// Config changed
    ConfigChanged(Config),
    /// Change game state
    #[cfg(not(feature = "macroquad-backend"))]
    StateTransition(Rc<RefCell<dyn GameState>>),
    #[cfg(feature = "macroquad-backend")]
    StateTransition(Box<dyn GameState>),
    /// Quit to desktop
    Quit,
}

impl<T: 'static + Debug> Event<T> {
    /// This allow construction of state transition events without worrying about the different
    /// types used by the two backends
    pub fn state_transition<S: 'static + GameState>(state: S) -> Self {
        state_transition(state)
    }
}

impl<T: 'static + Debug> Debug for Event<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Custom(event) => format!("Event::Custom({:?})", &event).fmt(f),
            Event::ConfigChanged(..) => format!("Event::ConfigChanged(Config)").fmt(f),
            Event::StateTransition(..) => format!("Event::StateTransition").fmt(f),
            Event::Quit => format!("Event::Quit").fmt(f),
        }
    }
}

/// This allow construction of state transition events without worrying about the different
/// types used by the two backends
pub fn state_transition<T: 'static + Debug, S: 'static + GameState>(state: S) -> Event<T> {
    #[cfg(not(feature = "macroquad-backend"))]
    return Event::StateTransition(Rc::new(RefCell::new(state)));
    #[cfg(feature = "macroquad-backend")]
    return Event::StateTransition(Box::new(state));
}
