use std::collections::HashMap;

use macroquad::{
    experimental::scene::{Handle, HandleUntyped, Node, RefMut},
    prelude::*,
};

use crate::{capabilities::NetworkReplicate, ParticleEmitters, Player};

pub struct ParticleController {
    owner: Handle<Player>,
    particle_id: String,
    start_delay: f32,
    amount: u32,
    interval: f32,
    is_can_be_interrupted: bool,
    is_looped: bool,

    timer: f32,
    particles_emitted: u32,
    is_emitting_started: bool,
    is_waiting_for_reset: bool,
}

impl ParticleController {
    fn reset(&mut self) {
        self.is_emitting_started = false;
        self.timer = 0.0;
        self.particles_emitted = 0;
        self.is_waiting_for_reset = false;
    }

    pub fn update(&mut self) {
        if self.is_can_be_interrupted || self.is_waiting_for_reset {
            self.reset();
        }
    }
}

pub struct ParticleControllers {
    pub active: HashMap<String, ParticleController>,
}

impl ParticleControllers {
    pub fn new() -> Self {
        ParticleControllers {
            active: HashMap::new(),
        }
    }

    pub fn spawn(
        &mut self,
        owner: Handle<Player>,
        particle_controller_id: String,
        particle_id: String,
        start_delay: f32,
        amount: u32,
        interval: f32,
        is_can_be_interrupted: bool,
        is_looped: bool,
    ) {
        let particle_controller = ParticleController {
            owner,
            particle_id,
            start_delay,
            amount,
            interval,
            is_can_be_interrupted,
            is_looped,
            timer: 0.0,
            particles_emitted: 0,
            is_emitting_started: false,
            is_waiting_for_reset: false,
        };

        self.active
            .insert(particle_controller_id, particle_controller);
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut controllers_to_delete: Vec<String> = Vec::new();

        for (key, controller) in node.active.iter_mut() {
            if controller.is_waiting_for_reset {
                continue;
            }

            controller.timer += get_frame_time();

            if controller.is_emitting_started {
                if controller.timer >= controller.interval {
                    controller.timer = 0.0;
                    controller.particles_emitted += 1;

                    {
                        let mut need_to_delete = true;
                        if let Some(player) = scene::try_get_node(controller.owner) {
                            if let Some(weapon) = &player.weapon {
                                let mut particles =
                                    scene::find_node_by_type::<ParticleEmitters>().unwrap();
                                    
                                let origin = player.get_weapon_mount_position()
                                    + weapon.get_effect_offset(!player.body.is_facing_right, false);

                                particles.spawn(&controller.particle_id, origin);

                                need_to_delete = false;
                            }
                        }

                        if need_to_delete {
                            controllers_to_delete.push(key.into());
                        }
                    }

                    if controller.particles_emitted == controller.amount {
                        if !controller.is_looped {
                            controller.is_waiting_for_reset = true;
                        } else {
                            controller.reset();
                        }
                    }
                }
            } else {
                if controller.timer >= controller.start_delay {
                    controller.is_emitting_started = true;
                    controller.timer = controller.interval;
                }
            }
        }

        for key in &controllers_to_delete {
            node.active.remove(key.into());
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
