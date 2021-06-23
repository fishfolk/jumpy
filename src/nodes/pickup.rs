use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, wait_seconds},
        scene::{self, RefMut},
    },
    prelude::*,
};

use crate::Resources;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ItemType {
    Gun = 1,
    Sword = 2,
}

pub struct Pickup {
    pub pos: Vec2,
    pub item_type: ItemType,
    visual_scale: f32,
}

impl Pickup {
    pub fn new(pos: Vec2, item_type: ItemType) -> Pickup {
        Pickup {
            pos,
            visual_scale: 1.0,
            item_type,
        }
    }
}

impl scene::Node for Pickup {
    fn ready(node: RefMut<Self>) {
        let handle = node.handle();

        start_coroutine(async move {
            let n = 25;
            for i in 0..n {
                // if player pick up the item real quick - the node may be already removed here
                if let Some(mut this) = scene::try_get_node(handle) {
                    this.visual_scale =
                        1.0 + (i as f32 / n as f32 * std::f32::consts::PI).sin() * 3.0;
                }

                next_frame().await;
            }
        });

        start_coroutine(async move {
            wait_seconds(10.).await;

            let n = 10;
            for _ in 0..n {
                if let Some(mut this) = scene::try_get_node(handle) {
                    this.visual_scale -= 1.0 / n as f32;
                }
                next_frame().await;
            }

            if let Some(this) = scene::try_get_node(handle) {
                this.delete();
            }
        });
    }

    fn draw(node: RefMut<Self>) {
        let resources = storage::get_mut::<Resources>();

        resources.tiled_map.spr_ex(
            "tileset",
            Rect::new(0.0 * 32.0, 6.0 * 32.0, 32.0, 32.0),
            Rect::new(
                node.pos.x - (32.0 * node.visual_scale - 32.) / 2.,
                node.pos.y - (32.0 * node.visual_scale - 32.) / 2.,
                32.0 * node.visual_scale,
                32.0 * node.visual_scale,
            ),
        );

        match node.item_type {
            ItemType::Gun => draw_texture_ex(
                resources.gun,
                node.pos.x,
                node.pos.y + 8.,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(0.0, 0.0, 64., 32.)),
                    dest_size: Some(vec2(32., 16.)),
                    ..Default::default()
                },
            ),
            ItemType::Sword => draw_texture_ex(
                resources.sword,
                node.pos.x + 4.,
                node.pos.y - 4.,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(195.0 + 5., 93.0 + 5., 65. - 10., 93. - 10.)),
                    dest_size: Some(vec2(32., 32.)),
                    ..Default::default()
                },
            ),
        }
    }
}
