use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    telemetry,
    prelude::*,
};

pub struct ScoreCounter {
    pub player_one: i32,
    pub player_two: i32,
}

// FIXME: Scaling and position of text should be constant regardless of screen size

impl ScoreCounter {
    pub const POSITION_Y_OFFSET: f32 = 90.0;

    pub fn new() -> ScoreCounter {
        ScoreCounter {
            player_one: 0,
            player_two: 0,
        }
    }
}

impl scene::Node for ScoreCounter {
    fn draw(_node: RefMut<Self>) {
        let pos = scene::find_node_by_type::<crate::nodes::Camera>()
            .unwrap()
            .macroquad_camera()
            .screen_to_world(vec2(screen_width() / 2.0, Self::POSITION_Y_OFFSET));

        draw_text(&format!("{} / {}", _node.player_one, _node.player_two),  pos.x, pos.y,40.0,WHITE);
    }
}
