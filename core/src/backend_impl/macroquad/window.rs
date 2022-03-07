use crate::Result;
use crate::window::{Window, WindowConfig};
use crate::video::Display;

#[derive(Default)]
pub(crate) struct WindowImpl {

}

pub fn create_window<D: Into<Option<Display>>>(title: &str, display: D, config: &WindowConfig) -> Result<Window> {
    Ok(WindowImpl::default().into())
}