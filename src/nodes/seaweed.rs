use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
        scene::{
            Node,
            RefMut,
        },
        collections::storage,
    },
    color,
    prelude::*,
};

use crate::{
    Resources,
    nodes::Player
};

pub struct Seaweed {
    pub sprite: AnimatedSprite,
    pub pos: Vec2,
}

impl Seaweed {
    pub const WIDTH: f32 = 32.0;
    pub const HEIGHT: f32 = 16.0;

    pub const SPEED_THRESHOLD: f32 = 100.0;
    pub const INCAPACITATE_DURATION: f32 = 3.0;

    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            16,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 2,
                    fps: 1,
                },
            ],
            false,
        );

        Seaweed{
            sprite,
            pos,
        }
    }
}

impl Node for Seaweed {
    fn fixed_update(node: RefMut<Seaweed>) {
        let hitbox = Rect::new(node.pos.x, node.pos.y, Seaweed::WIDTH, Seaweed::HEIGHT);
        for mut player in scene::find_nodes_by_type::<Player>() {
            if hitbox.overlaps(&player.get_hitbox()) {
                if player.body.speed.x >= Seaweed::SPEED_THRESHOLD || player.body.speed.x <= -Seaweed::SPEED_THRESHOLD {
                    player.incapacitate(Seaweed::INCAPACITATE_DURATION, true);
                }
            }
        }
    }

    fn draw(mut node: RefMut<Seaweed>) {
        node.sprite.update();

        let resources = storage::get_mut::<Resources>();

        draw_texture_ex(
            resources.seaweed,
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
