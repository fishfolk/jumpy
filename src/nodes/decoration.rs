use macroquad::{
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{self, RefMut},
    },
    prelude::*,
};

use crate::Resources;

pub struct Decoration {
    pos: Vec2,
    sprite: AnimatedSprite,
}

impl Decoration {
    pub fn new(pos: Vec2, gid: u32) -> Decoration {
        let mut sprite = AnimatedSprite::new(
            48,
            51,
            &[
                Animation {
                    name: "grass".to_string(),
                    row: 0,
                    frames: 5,
                    fps: 8,
                },
                Animation {
                    name: "bowls".to_string(),
                    row: 1,
                    frames: 5,
                    fps: 6,
                },
            ],
            true,
        );
        if gid == 50 {
            sprite.set_animation(0);
        } else {
            sprite.set_animation(1);
        }
        Decoration { pos, sprite }
    }
}

impl scene::Node for Decoration {
    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get_mut::<Resources>();

        draw_texture_ex(
            resources.decorations,
            node.pos.x,
            node.pos.y - 51.,
            WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                ..Default::default()
            },
        );
    }
}
