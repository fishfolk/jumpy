use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use std::fmt::Debug;

use crate::event::Event;

pub struct DefaultEventHandler;

pub type DefaultCustomEvent = ();

impl<E: 'static + Debug> EventHandler<E> for DefaultEventHandler {}

pub trait EventHandler<E: 'static + Debug> {
    fn handle(&mut self, _event: &glutin::event::Event<Event<E>>, _control_flow: &mut ControlFlow) {
    }

    fn handle_custom(&mut self, _event: &E, _control_flow: &mut ControlFlow) {}
}
