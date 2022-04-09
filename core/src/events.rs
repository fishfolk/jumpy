//! These events are meant to trigger actions in the main loop.
//! This might seem excessive, for now, but in the future we want to be able to do things like
//! jumping between game modes, for example, like starting a test game with a map we are editing
//! in the editor, without having to exit to main menu, select game mode, select map, etc.

use crate::state::GameState;
use std::sync::{Arc, Mutex};

static mut APPLICATION_EVENTS: Option<Arc<Mutex<Vec<GameEvent>>>> = None;

fn get_event_queue() -> Arc<Mutex<Vec<GameEvent>>> {
    unsafe { APPLICATION_EVENTS.get_or_insert(Arc::new(Mutex::new(Vec::new()))) }.clone()
}

pub fn dispatch_game_event(event: GameEvent) {
    unsafe { get_event_queue() }.lock().unwrap().push(event);
}

pub fn iter_events() -> GameEventIterator {
    GameEventIterator::new()
}

/// This holds all the event types
pub enum GameEvent {
    /// Change game mode
    StateTransition(Box<dyn GameState>),
    /// Reload resources
    ReloadResources,
    /// Quit to desktop
    Quit,
}

impl GameEvent {
    pub fn dispatch(self) {
        dispatch_game_event(self);
    }
}

pub fn dispatch_event(event: GameEvent) {
    event.dispatch();
}

/// This iterates over all the events in the event queue
#[derive(Default)]
pub struct GameEventIterator {}

impl GameEventIterator {
    pub fn new() -> Self {
        GameEventIterator {}
    }
}

impl Iterator for GameEventIterator {
    type Item = GameEvent;

    fn next(&mut self) -> Option<Self::Item> {
        get_event_queue().lock().unwrap().pop()
    }
}
