use std::collections::HashMap;

use glow::Context;
use winit::window::WindowId;

use crate::window::Window;

static mut GL_CONTEXTS: Option<HashMap<WindowId, Context>> = None;

fn gl_contexts() -> &'static mut HashMap<WindowId, Context> {
    unsafe { GL_CONTEXTS.get_or_insert_with(|| HashMap::new()) }
}

pub fn gl_context(window: Window) -> &'static mut Context {
    let id = window.id();

    if !gl_contexts().contains_key(&id) {
        let context = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };

        gl_contexts().insert(window.id(), context);
    }

    gl_contexts().get_mut(&id).unwrap()
}
