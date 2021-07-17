use macroquad::{
    audio::play_sound_once,
    experimental::{
      collections::storage,
      scene::RefMut,
      animation::{
          AnimatedSprite,
          Animation,
      },
    },
    color,
    prelude::*,
};

use crate::Resources;

pub struct Sproinger {
    sprite: AnimatedSprite,
    pos: Vec2,
    has_sproinged: bool,
    time_since_sproing: f32,
}

impl Sproinger {
    pub const TRIGGER_WIDTH: f32 = 32.0;
    pub const TRIGGER_HEIGHT: f32 = 8.0;
    pub const FORCE: f32 = 600.0;
    pub const COOLDOWN: f32 = 0.5;
    pub const STOPPED_THRESHOLD: f32 = 0.01;

    pub fn new(pos: Vec2) -> Self {
        let sprite = AnimatedSprite::new(
            32,
            32,
            &[
                Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 1,
                },
                Animation {
                    name: "sproing".to_string(),
                    row: 0,
                    frames: 3,
                    fps: 10,
                },
                Animation {
                    name: "desproing".to_string(),
                    row: 1,
                    frames: 3,
                    fps: 10,
                },
            ],
            false,
        );

        Sproinger {
            sprite,
            pos,
            has_sproinged: false,
            time_since_sproing: 0.0,
        }
    }

    pub fn sproing(&mut self) {
        self.has_sproinged = true;
        self.time_since_sproing = 0.0;
        // self.sprite.set_animation(1);
        // self.sprite.playing = true;

        let resources = storage::get_mut::<Resources>();
        play_sound_once(resources.jump_sound);
    }
}

impl scene::Node for Sproinger {
    fn fixed_update(mut node: RefMut<Self>) {
        if node.has_sproinged {
            node.time_since_sproing += get_frame_time();
            if node.time_since_sproing >= Self::COOLDOWN {
                node.has_sproinged = false;
                node.time_since_sproing = 0.0;
            }
        }

        if !node.has_sproinged {
            let sproinger_rect = Rect::new(
                node.pos.x, // - (Self::TRIGGER_WIDTH / 2.0),
                node.pos.y + (node.sprite.frame().dest_size.y - Self::TRIGGER_HEIGHT),
                Self::TRIGGER_WIDTH,
                Self::TRIGGER_HEIGHT,
            );

            for mut player in scene::find_nodes_by_type::<crate::nodes::Player>() {
                if player.body.speed.length() > Self::STOPPED_THRESHOLD {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        player.body.pos.x,
                        player.body.pos.y,
                        20.0,
                        64.0,
                    ));
                    if !intersect.is_none() {
                        player.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut muscet in scene::find_nodes_by_type::<crate::nodes::Muscet>() {
                if muscet.body.speed.length() > Self::STOPPED_THRESHOLD {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        muscet.body.pos.x,
                        muscet.body.pos.y,
                        92.0,
                        32.0,
                    ));
                    if !intersect.is_none() {
                        muscet.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut sword in scene::find_nodes_by_type::<crate::nodes::Sword>() {
                if sword.body.speed.length() > Self::STOPPED_THRESHOLD {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        sword.body.pos.x,
                        sword.body.pos.y,
                        64.0,
                        32.0,
                    ));
                    if !intersect.is_none() {
                        sword.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut mines in scene::find_nodes_by_type::<crate::nodes::Mines>() {
                if mines.body.speed.length() > Self::STOPPED_THRESHOLD {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        mines.body.pos.x,
                        mines.body.pos.y,
                        16.0,
                        32.0,
                    ));
                    if !intersect.is_none() {
                        mines.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut mines in scene::find_nodes_by_type::<crate::nodes::ArmedMines>() {
                for mine in &mut mines.mines {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        mine.body.pos.x,
                        mine.body.pos.y,
                        16.0,
                        32.0,
                    ));
                    if !intersect.is_none() {
                        mine.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut grenades in scene::find_nodes_by_type::<crate::nodes::Grenades>() {
                if grenades.body.speed.length() > Self::STOPPED_THRESHOLD {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        grenades.body.pos.x,
                        grenades.body.pos.y,
                        16.0,
                        96.0,
                    ));
                    if !intersect.is_none() {
                        grenades.body.speed.y = -Self::FORCE;
                        node.sproing();
                    }
                }
            }

            for mut grenades in scene::find_nodes_by_type::<crate::nodes::ArmedGrenades>() {
                for grenade in &mut grenades.grenades {
                    let intersect = sproinger_rect.intersect(Rect::new(
                        grenade.body.pos.x,
                        grenade.body.pos.y,
                        16.0,
                        32.0,
                    ));
                    if !intersect.is_none() {
                        grenade.body.speed.y = -Self::FORCE;
                        node.sproing();
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
