use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{Node, RefMut},
    },
    prelude::*,
};

use crate::{nodes::Player, Resources};

pub struct Shoes {
    sprite: AnimatedSprite,
    pos: Vec2,
}

impl Shoes {
    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            32,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        Shoes { sprite, pos }
    }
}

impl Node for Shoes {
    fn fixed_update(node: RefMut<Self>) {
        let shoes_rect = Rect::new(node.pos.x, node.pos.y, 30.0, 30.0);
        let mut picked_up = false;
        for mut player in scene::find_nodes_by_type::<Player>() {
            if !player.can_head_boink {
                let player_rect = Rect::new(player.body.pos.x, player.body.pos.y, 30.0, 54.0);
                if player_rect.overlaps(&shoes_rect) {
                    player.can_head_boink = true;
                    picked_up = true;
                }
            }
        }
        if picked_up {
            node.delete();
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();

        let resources = storage::get_mut::<Resources>();

        draw_texture_ex(
            resources.shoes,
            node.pos.x,
            node.pos.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(node.sprite.frame().source_rect),
                dest_size: Some(node.sprite.frame().dest_size),
                flip_x: false,
                rotation: 0.0,
                ..Default::default()
            },
        );
    }
}
