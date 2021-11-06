use std::collections::HashMap;

use macroquad::{
    experimental::scene::{Handle, HandleUntyped, Node, RefMut},
    prelude::*,
};

use crate::{
    capabilities::NetworkReplicate,
    components::{ParticleController, ParticleControllerParams},
    Player,
};

pub struct PlayerParticleController {
    handler: Handle<Player>,
    particle: ParticleController,
}

#[derive(Default)]
pub struct ParticleControllers {
    pub active: HashMap<String, PlayerParticleController>,
}

impl ParticleControllers {
    pub fn spawn_or_update(&mut self, owner: Handle<Player>, params: &ParticleControllerParams) {
        if let Some(player) = scene::try_get_node(owner) {
            let hash = player.id.to_string() + &params.id;

            if let Some(controller) = self.active.get_mut(&hash) {
                if let Some(weapon) = &player.weapon {
                    controller
                        .particle
                        .update(weapon.get_effect_position(&player), true);
                }
            } else {
                let player_particle_controller = PlayerParticleController {
                    handler: owner,
                    particle: ParticleController::new(params.clone()),
                };

                self.active.insert(hash, player_particle_controller);
            }
        }
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut controllers_to_delete: Vec<String> = Vec::new();

        for (key, controller) in node.active.iter_mut() {
            let mut need_to_delete = true;

            if let Some(player) = scene::try_get_node(controller.handler) {
                if let Some(weapon) = &player.weapon {
                    controller
                        .particle
                        .update(weapon.get_effect_position(&player), false);

                    need_to_delete = false;
                }
            }

            if need_to_delete {
                controllers_to_delete.push(key.into());
            }
        }

        for key in &controllers_to_delete {
            node.active.remove(key);
        }
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<ParticleControllers>();
            ParticleControllers::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}

impl Node for ParticleControllers {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }
}
