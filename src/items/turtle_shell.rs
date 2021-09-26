use macroquad::{
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        scene::{HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{nodes::Player, Resources};

pub struct TurtleShell {
    sprite: AnimatedSprite,
    pos: Vec2,
}

impl TurtleShell {
    pub fn spawn(pos: Vec2) -> HandleUntyped {
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

        scene::add_node(TurtleShell { sprite, pos }).untyped()
    }
}

impl Node for TurtleShell {
    fn fixed_update(node: scene::RefMut<Self>) {
        let turtleshell_rect = Rect::new(node.pos.x, node.pos.y, 30.0, 30.0);
        let mut picked_up = false;

        for mut player in scene::find_nodes_by_type::<Player>() {
            let player_rect = Rect::new(player.body.pos.x, player.body.pos.y, 30.0, 54.0);
            if player_rect.overlaps(&turtleshell_rect) {
                // give the player two armor on pickup
                player.back_armor += 2;
                picked_up = true;
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
            resources.turtleshell,
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
        )
    }
}
