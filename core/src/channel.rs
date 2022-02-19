//! This is just a wrapper for `std::sync::mpsc::channel` that is a bit tidier than the tuple
//! provided by the std

use std::sync::mpsc::{Receiver, Sender};

pub struct Channel<T, R> {
    pub tx: Sender<T>,
    pub rx: Receiver<R>,
}

impl<T, R> Channel<T, R> {
    pub fn new(tx: Sender<T>, rx: Receiver<R>) -> Self {
        Channel { tx, rx }
    }
}
