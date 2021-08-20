use macroquad::{
    audio::play_sound_once,
    color,
    experimental::{
        animation::{AnimatedSprite, Animation},
        collections::storage,
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::{self, Handle, HandleUntyped, Lens, RefMut},
    },
    prelude::*,
};

use crate::{nodes::player::PhysicsBody, Resources};

pub type Sproingable = (HandleUntyped, Lens<PhysicsBody>, Vec2);

pub struct Sproinger {
    sprite: AnimatedSprite,
    pos: Vec2,
    has_sproinged: bool,
}

impl Sproinger {
    pub const TRIGGER_WIDTH: f32 = 32.0;
    pub const TRIGGER_HEIGHT: f32 = 8.0;
    pub const FORCE: f32 = 1100.0;
    pub const STOPPED_THRESHOLD: f32 = 0.01;

    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            31,
            20,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "sproing".to_string(),
                    row: 1,
                    frames: 2,
                    fps: 10,
                },
                Animation {
                    name: "desproing".to_string(),
                    row: 1,
                    frames: 2,
                    fps: 10,
                },
            ],
            false,
        );

        Sproinger {
            sprite,
            pos,
            has_sproinged: false,
        }
    }

    fn animate(node_handle: Handle<Sproinger>) -> Coroutine {
        let coroutine = async move {
            {
                let mut node = scene::get_node(node_handle);
                node.sprite.set_animation(1);
            }
            for i in 0..2 {
                {
                    let mut node = scene::get_node(node_handle);
                    if node.sprite.current_animation() != 1 {
                        return;
                    }
                    node.sprite.set_frame(i);
                }
                wait_seconds(0.08).await;
            }
            {
                let mut node = scene::get_node(node_handle);
                node.sprite.set_animation(2);
            }
            for i in 0..2 {
                {
                    let mut node = scene::get_node(node_handle);
                    if node.sprite.current_animation() != 2 {
                        return;
                    }
                    node.sprite.set_frame(i);
                }
                wait_seconds(0.08).await;
            }
            wait_seconds(0.5).await;

            {
                let mut node = scene::get_node(node_handle);
                node.has_sproinged = false;
                node.sprite.set_animation(0);
            }
        };
        start_coroutine(coroutine)
    }
}

impl scene::Node for Sproinger {
    fn fixed_update(mut node: RefMut<Self>) {
        if !node.has_sproinged {
            let sproinger_rect = Rect::new(
                node.pos.x, // - (Self::TRIGGER_WIDTH / 2.0),
                node.pos.y + (node.sprite.frame().dest_size.y - Self::TRIGGER_HEIGHT),
                Self::TRIGGER_WIDTH,
                Self::TRIGGER_HEIGHT,
            );

            for (_actor, mut body_lens, size) in scene::find_nodes_with::<Sproingable>() {
                if body_lens.get().is_some() {
                    let body = body_lens.get().unwrap();
                    if body.speed.length() > Self::STOPPED_THRESHOLD {
                        let intersect = sproinger_rect
                            .intersect(Rect::new(body.pos.x, body.pos.y, size.x, size.y));
                        if intersect.is_some() {
                            let resources = storage::get_mut::<Resources>();
                            play_sound_once(resources.jump_sound);

                            body.speed.y = -Self::FORCE;
                            node.has_sproinged = true;
                            // self.sprite.set_animation(1);
                            // self.sprite.playing = true;
                            Sproinger::animate(node.handle());
                        }
                    }
                }
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        node.sprite.update();
        // if node.has_sproinged {
        //     if node.sprite.current_animation() == 1 {
        //         node.sprite.set_animation(2);
        //         node.sprite.playing = true;
        //     }
        // }

        let resources = storage::get_mut::<Resources>();

        draw_texture_ex(
            resources.sproinger,
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
