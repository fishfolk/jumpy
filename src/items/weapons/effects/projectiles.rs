use macroquad::{
    experimental::{
        collections::storage,
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::{capabilities::NetworkReplicate, GameWorld, Player};

// TODO: Performance test this and reduce complexity as needed
struct Projectile {
    owner: Handle<Player>,
    origin: Vec2,
    position: Vec2,
    velocity: Vec2,
    range: f32,
    size: f32,
    color: Color,
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
        origin: Vec2,
        velocity: Vec2,
        range: f32,
        size: f32,
        color: Color,
    ) {
        self.active.push(Projectile {
            owner,
            origin,
            position: origin,
            velocity,
            range,
            size,
            color,
        });
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        'projectiles: while i < node.active.len() {
            let projectile = &mut node.active[i];
            projectile.position += projectile.velocity;

            {
                let distance = projectile.position.distance(projectile.origin);
                if distance > projectile.range {
                    node.active.remove(i);
                    continue 'projectiles;
                }

                let world = storage::get::<GameWorld>();

                if world.map.is_collision_at(projectile.position, true) {
                    node.active.remove(i);
                    continue 'projectiles;
                }
            }

            let mut collider = None;
            if projectile.size > 1.5 {
                let circle = Circle::new(
                    projectile.position.x,
                    projectile.position.y,
                    projectile.size,
                );
                collider = Some(circle);
            }

            // Borrow owner so that it is excluded from the following iteration and hit check
            let _owner = scene::try_get_node(projectile.owner);

            for mut player in scene::find_nodes_by_type::<Player>() {
                let hitbox = player.get_collider();
                let has_collision = if let Some(circle) = &collider {
                    circle.overlaps_rect(&hitbox)
                } else {
                    hitbox.contains(projectile.position)
                };

                if has_collision {
                    let direction = projectile.position.x > player.body.pos.x;
                    player.kill(direction);

                    node.active.remove(i);
                    continue 'projectiles;
                }
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

    fn draw(node: RefMut<Self>) {
        for projectile in &node.active {
            draw_circle(
                projectile.position.x,
                projectile.position.y,
                projectile.size,
                projectile.color,
            )
        }
    }
}

pub fn default_projectile_color() -> Color {
    Color::new(1.0, 1.0, 0.8, 1.0)
}
