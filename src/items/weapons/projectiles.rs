use macroquad::{
    experimental::{
        scene::{
            Node,
            Handle,
            HandleUntyped,
            RefMut,
        },
    },
    prelude::*,
};

use crate::{
    Player,
    capabilities::NetworkReplicate,
};

pub struct Projectile {
    owner: Handle<Player>,
    position: Vec2,
    velocity: Vec2,
    ttl: Option<f32>,
    ttl_timer: f32,
}

pub struct Projectiles {
    active: Vec<Projectile>,
}

impl Projectiles {
    pub fn new() -> Self {
        Projectiles {
            active: Vec::new(),
        }
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        'projectiles: while i < node.active.len() {
            let projectile = &mut node.active[i];
            if let Some(ttl) = projectile.ttl {
                projectile.ttl_timer += get_frame_time();
                if projectile.ttl_timer >= ttl {
                    node.active.remove(i);
                    continue 'projectiles;
                }
            }

            projectile.position += projectile.velocity;

            // Borrow owner so that it is excluded from the following iteration and hit checking
            let _owner = scene::try_get_node(projectile.owner);

            for mut player in scene::find_nodes_by_type::<Player>() {
                let hitbox = player.get_hitbox();
                if hitbox.contains(projectile.position) {
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
    fn ready(mut node: RefMut<Self>) where Self: Sized {
        node.provides(Self::network_capabilities());
    }
}