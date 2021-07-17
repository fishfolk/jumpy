use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    prelude::*,
};

use crate::Resources;

pub struct ScoreCounter {
    player0_lifes: i32,
    player1_lifes: i32,
}

impl ScoreCounter {
    pub fn new() -> ScoreCounter {
        ScoreCounter {
            player0_lifes: 3,
            player1_lifes: 3,
        }
    }

    pub fn count_loss(&mut self, id: i32) {
        let mut resources = storage::get_mut::<Resources>();

        let w = 76. / 2.;

        if id == 0 {
            resources
                .life_explosion_fxses
                .spawn(vec2(w * self.player0_lifes as f32, 50.));

            self.player0_lifes -= 1;
        }

        if id == 1 {
            resources.life_explosion_fxses.spawn(vec2(
                screen_width() - w - w * self.player1_lifes as f32,
                50.,
            ));

            self.player1_lifes -= 1;
        }
    }
}

impl scene::Node for ScoreCounter {
    fn draw(node: RefMut<Self>) {
        let resources = storage::get::<Resources>();

        push_camera_state();
        set_default_camera();

        let w = 76. / 2.;
        let h = 66.0 / 2.;
        for i in 0..node.player0_lifes {
            draw_texture_ex(
                resources.whale,
                (w + 2.) * i as f32,
                0.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, 76.0, 66.0)),
                    dest_size: Some(vec2(w, h)),
                    flip_x: false,
                    ..Default::default()
                },
            );
        }

        for i in 0..node.player1_lifes {
            draw_texture_ex(
                resources.whale_red,
                screen_width() - (w + 2.) * (i + 1) as f32,
                0.0,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, 76.0, 66.0)),
                    dest_size: Some(vec2(w, h)),
                    flip_x: true,
                    ..Default::default()
                },
            );
        }

        pop_camera_state();
    }
}
