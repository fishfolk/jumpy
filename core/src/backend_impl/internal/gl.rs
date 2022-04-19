use glow::Context;
use glutin::window::Window;
use glutin::PossiblyCurrent;

static mut GL_CONTEXT: Option<Context> = None;

pub fn gl_context() -> &'static Context {
    unsafe {
        GL_CONTEXT.as_ref().unwrap_or_else(|| panic!("ERROR: Attempted to get gl context but none has been created! Did you call `create_window`?"))
    }
}

pub fn create_gl_context(
    window: &glutin::ContextWrapper<PossiblyCurrent, Window>,
) -> &'static Context {
    unsafe {
        let context = Context::from_loader_function(|s| window.get_proc_address(s));
        GL_CONTEXT = Some(context);
    };

    gl_context()
}
