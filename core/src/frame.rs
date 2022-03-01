use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "no-macroquad-main")] {
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        pub enum FramePhase {
            None,
            Update,
            FixedUpdate,
            Draw,
        }

        static mut CURRENT_FRAME_PHASE: FramePhase = FramePhase::None;
        static mut CURRENT_DELTA_TIME: f32 = 0.0;

        pub fn get_delta_time() -> f32 {
            unsafe { CURRENT_DELTA_TIME }
        }

        pub fn begin_frame(dt: f32) {
            unsafe {
                CURRENT_FRAME_PHASE = FramePhase::Update;
                CURRENT_DELTA_TIME = dt;
            }
        }

        pub fn begin_fixed_update(dt: f32) {
            unsafe {
                CURRENT_FRAME_PHASE = FramePhase::FixedUpdate;
                CURRENT_DELTA_TIME = dt;
            }
        }

        pub fn begin_draw(dt: f32) {
            unsafe {
                CURRENT_FRAME_PHASE = FramePhase::Draw;
                CURRENT_DELTA_TIME = dt;
            }
        }

        pub fn end_frame() {
            unsafe {
                CURRENT_FRAME_PHASE = FramePhase::None;
                CURRENT_DELTA_TIME = 0.0;
            }
        }

        pub fn get_current_phase() -> FramePhase {
            unsafe { CURRENT_FRAME_PHASE }
        }
    } else {
        pub fn get_delta_time() -> f32 { macroquad::time::get_frame_time() }
    }
}