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

pub struct Flippers {
    sprite: AnimatedSprite,
    pos: Vec2,
}

impl Flippers {
    pub fn spawn(pos: Vec2) -> HandleUntyped {
        let sprite = AnimatedSprite::new(
            65,
            45,
            &[Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 1,
                fps: 1,
            }],
            false,
        );

        scene::add_node(Flippers { sprite, pos }).untyped()
    }
}

impl Node for Flippers {
    fn fixed_update(node: scene::RefMut<Self>) {
        let flippers_rect = Rect::new(node.pos.x, node.pos.y, 30.0, 30.0);
        let mut picked_up = false;

        for mut player in scene::find_nodes_by_type::<Player>() {
            let player_rect = Rect::new(player.body.pos.x, player.body.pos.y, 30.0, 54.0);
            if player_rect.overlaps(&flippers_rect) && !player.can_head_boink {
                player.can_extra_jump = true;
                player.can_head_boink = true;
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
            resources.items_textures["flippers/flippers_item"],
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
