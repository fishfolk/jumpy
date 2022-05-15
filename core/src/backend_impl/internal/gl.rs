use glow::{Context, HasContext};
use glutin::window::Window;
use glutin::PossiblyCurrent;
use std::rc::Rc;

static mut GL_CONTEXT: Option<Rc<Context>> = None;

pub fn gl_context() -> Rc<Context> {
    unsafe {
        GL_CONTEXT.as_ref().unwrap_or_else(|| panic!("ERROR: Attempted to get gl context but none has been created! Did you call `create_window`?")).clone()
    }
}

pub fn init_gl_context(window: &glutin::ContextWrapper<PossiblyCurrent, Window>) -> Rc<Context> {
    unsafe {
        let gl = Context::from_loader_function(|addr| window.get_proc_address(addr) as *const _);

        //gl.enable(glow::FRAMEBUFFER_SRGB);
        //gl.enable(glow::BLEND);
        //gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        #[cfg(debug_assertions)]
        {
            let version = gl.version();
            println!(
                "OpenGL: {}.{} {}, is_embedded: {}",
                version.major, version.minor, version.vendor_info, version.is_embedded
            );
        }

        GL_CONTEXT = Some(Rc::new(gl));
    };

    gl_context()
}
