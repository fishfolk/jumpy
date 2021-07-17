use macroquad::{
    experimental::{
        scene::{self, RefMut},
    },
    prelude::*,
};

pub struct ScoreCounter {
    pub player_one: i32,
    pub player_two: i32,
}

// FIXME: Scaling and position of text should be constant regardless of screen size

impl ScoreCounter {
    pub const POSITION_Y_OFFSET: f32 = 90.0;
    pub const FONT_SIZE: f32 = 72.0;

    pub fn new() -> ScoreCounter {
        ScoreCounter {
            player_one: 0,
            player_two: 0,
        }
    }
}

impl scene::Node for ScoreCounter {
    fn draw(_node: RefMut<Self>) {
        push_camera_state();
        set_default_camera();
        draw_text(
            &format!("{} / {}", _node.player_one, _node.player_two),
            screen_width() / 2.0,
            Self::POSITION_Y_OFFSET,
            Self::FONT_SIZE,
            WHITE,
        );
        pop_camera_state();
    }
}
