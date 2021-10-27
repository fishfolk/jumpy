//! These events are meant to trigger actions in the main loop.
//! This might seem excessive, for now, but in the future we want to be able to do things like
//! jumping between game modes, for example, like starting a test game with a map we are editing
//! in the editor, without having to exit to main menu, select game mode, select map, etc.

static mut APPLICATION_EVENTS: Option<Vec<ApplicationEvent>> = None;

unsafe fn get_event_queue() -> &'static mut Vec<ApplicationEvent> {
    APPLICATION_EVENTS.get_or_insert(Vec::new())
}

pub fn dispatch_application_event(event: ApplicationEvent) {
    unsafe { get_event_queue() }.push(event);
}

pub fn iter_events() -> ApplicationEventIterator {
    ApplicationEventIterator::new()
}

/// This holds all the event types
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ApplicationEvent {
    /// Exit to main menu
    MainMenu,
    /// Quit to desktop
    Quit,
}

impl ApplicationEvent {
    pub fn dispatch(self) {
        dispatch_application_event(self);
    }
}

/// This iterates over all the events in the event queue
#[derive(Default)]
pub struct ApplicationEventIterator {}

impl ApplicationEventIterator {
    pub fn new() -> Self {
        ApplicationEventIterator {}
    }
}

impl Iterator for ApplicationEventIterator {
    type Item = ApplicationEvent;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { get_event_queue() }.pop()
    }
}
