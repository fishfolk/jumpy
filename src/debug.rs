#[cfg(debug_assertions)]
mod implementation {
    static mut IS_DEBUG_DRAW_ENABLED: bool = true;

    pub fn is_debug_draw_enabled() -> bool {
        unsafe { IS_DEBUG_DRAW_ENABLED }
    }

    pub fn enable_debug_draw() {
        unsafe { IS_DEBUG_DRAW_ENABLED = true };
    }

    pub fn disable_debug_draw() {
        unsafe { IS_DEBUG_DRAW_ENABLED = false };
    }

    pub fn toggle_debug_draw() {
        unsafe { IS_DEBUG_DRAW_ENABLED = !IS_DEBUG_DRAW_ENABLED }
    }
}

#[cfg(debug_assertions)]
pub use implementation::*;
