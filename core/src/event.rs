pub use crate::backend_impl::event::*;

use std::fmt::{Debug, Formatter};

use crate::config::Config;
use crate::prelude::GameState;

pub type DefaultCustomEvent = ();

/// This holds all the event types
pub enum Event<E: 'static + Debug> {
    /// Custom event
    Custom(E),
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

impl<E: 'static + Debug> Event<E> {
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
            Event::ConfigChanged(..) => "Event::ConfigChanged(Config)".to_string().fmt(f),
            Event::StateTransition(..) => "Event::StateTransition".to_string().fmt(f),
            Event::Quit => "Event::Quit".to_string().fmt(f),
        }
    }
}

/// This allow construction of state transition events without worrying about the different
/// types used by the two backends
pub fn state_transition<E: 'static + Debug, S: 'static + GameState>(state: S) -> Event<E> {
    #[cfg(not(feature = "macroquad-backend"))]
    return Event::StateTransition(Rc::new(RefCell::new(state)));
    #[cfg(feature = "macroquad-backend")]
    return Event::StateTransition(Box::new(state));
}
