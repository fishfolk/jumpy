use macroquad::{
    experimental::{
        collections::storage,
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    capabilities::NetworkReplicate,
    components::{Sprite, SpriteParams},
    json,
    nodes::ParticleEmitters,
    GameWorld, Player,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectileKind {
    Circle {
        radius: f32,
        #[serde(with = "json::ColorDef")]
        color: Color,
    },
    Rect {
        width: f32,
        height: f32,
        #[serde(with = "json::ColorDef")]
        color: Color,
    },
    Sprite {
        #[serde(rename = "sprite")]
        params: Option<SpriteParams>,
    },
}

// TODO: Performance test this and reduce complexity as needed
struct Projectile {
    owner: Handle<Player>,
    kind: ProjectileKind,
    origin: Vec2,
    position: Vec2,
    velocity: Vec2,
    range: f32,
    sprite: Option<Sprite>,
}

pub struct Projectiles {
    active: Vec<Projectile>,
}

impl Projectiles {
    pub fn new() -> Self {
        Projectiles { active: Vec::new() }
    }

    pub fn spawn(
        &mut self,
        owner: Handle<Player>,
        mut kind: ProjectileKind,
        origin: Vec2,
        velocity: Vec2,
        range: f32,
    ) {
        let sprite = if let ProjectileKind::Sprite { params } = &mut kind {
            let params = params.take().unwrap();
            Some(Sprite::new(params))
        } else {
            None
        };

        self.active.push(Projectile {
            owner,
            kind,
            origin,
            position: origin,
            velocity,
            range,
            sprite,
        });
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        while i < node.active.len() {
            let projectile = &mut node.active[i];
            projectile.position += projectile.velocity;

            let mut is_hit = false;

            {
                let distance = projectile.position.distance(projectile.origin);
                if distance > projectile.range {
                    is_hit = true;
                }

                if !is_hit {
                    let world = storage::get::<GameWorld>();

                    is_hit = world.collision_world.solid_at(projectile.position);
                }
            }

            if !is_hit {
                // Borrow owner so that it is excluded from the following iteration and hit check
                let _owner = scene::try_get_node(projectile.owner);

                for mut player in scene::find_nodes_by_type::<Player>() {
                    let hitbox = player.get_collider();

                    if hitbox.contains(projectile.position) {
                        let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                        particles.spawn("hit", projectile.position);

                        let direction = projectile.position.x > player.body.position.x;
                        player.kill(direction);

                        is_hit = true;
                        break;
                    }
                }
            }

            if is_hit {
                node.active.remove(i);
                continue;
            }

            i += 1;
        }
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<Projectiles>();
            Projectiles::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}

impl Node for Projectiles {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }

    fn draw(mut node: RefMut<Self>) {
        for projectile in &mut node.active {
            match projectile.kind.clone() {
                ProjectileKind::Circle { radius, color } => {
                    draw_circle(projectile.position.x, projectile.position.y, radius, color)
                }
                ProjectileKind::Rect {
                    width,
                    height,
                    color,
                } => draw_rectangle(
                    projectile.position.x,
                    projectile.position.y,
                    width,
                    height,
                    color,
                ),
                ProjectileKind::Sprite { .. } => {
                    let sprite = projectile.sprite.as_ref().unwrap();
                    let flip_x = projectile.velocity.x < 0.0;
                    sprite.draw(projectile.position, 0.0, flip_x, false);
                }
            }
        }
    }
}
