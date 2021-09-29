use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, Handle},
    },
    math::{vec2, Rect},
};

use crate::{components::PhysicsBody, nodes::Player, Resources, GameWorld};

#[derive(Default)]
pub struct ThrowableItem {
    pub owner: Option<Handle<Player>>,
}

impl ThrowableItem {
    pub fn thrown(&self) -> bool {
        self.owner.is_none()
    }

    pub fn throw(&mut self, body: &mut PhysicsBody, force: bool) {
        self.owner = None;

        if force {
            body.speed = if body.facing {
                vec2(600., -200.)
            } else {
                vec2(-600., -200.)
            };
        } else {
            body.angle = 3.5;
        }

        let mut world = storage::get_mut::<GameWorld>();

        world
            .collision_world
            .set_actor_position(body.collider, body.pos);
    }

    pub fn update(&mut self, body: &mut PhysicsBody, disarm: bool) {
        if self.thrown() {
            body.update();
            body.update_throw();

            if disarm && !body.on_ground {
                let hitbox = Rect::new(body.pos.x, body.pos.y, body.size.x, body.size.y);
                for mut player in scene::find_nodes_by_type::<Player>() {
                    if hitbox.overlaps(&player.get_hitbox()) {
                        if let Some(weapon) = player.weapon.as_mut() {
                            use crate::capabilities::WeaponTrait;

                            weapon.throw(false);
                            player.weapon = None;
                        }
                    }
                }
            }
        }
    }
}
