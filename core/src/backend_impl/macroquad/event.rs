use crate::events::Event;

static mut EVENTS: Vec<Event> = Vec::new();

pub fn dispatch_event(event: Event) {
    unsafe {
        EVENTS.push(event);
    }
}

pub fn iter_events() -> EventIterator {
    EventIterator::new()
}

/// This iterates over all the events in the event queue
#[derive(Default)]
pub struct EventIterator {}

impl EventIterator {
    pub fn new() -> Self {
        EventIterator {}
    }
}

impl Iterator for EventIterator {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe { EVENTS.pop() }
    }
}
