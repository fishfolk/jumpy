use ff_core::ecs::{Entity, World};

use serde::{Deserialize, Serialize};

use crate::effects::passive::PassiveEffectDamageBlockKind;
use crate::Item;
use ff_core::prelude::*;
use ff_core::Result;

use crate::player::{Player, PlayerState};

#[derive(Default)]
pub struct PlayerEventQueue {
    pub queue: Vec<PlayerEvent>,
}

impl PlayerEventQueue {
    pub fn new() -> Self {
        PlayerEventQueue { queue: Vec::new() }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageDirection {
    Back,
    Front,
}

impl Default for DamageDirection {
    fn default() -> Self {
        DamageDirection::Front
    }
}

#[derive(Clone)]
pub enum PlayerEvent {
    Update {
        delta_time: f32,
    },
    ReceiveDamage {
        is_from_left: bool,
        damage_from: Option<Entity>,
    },
    GiveDamage {
        damage_to: Option<Entity>,
    },
    DamageBlocked {
        is_from_left: bool,
    },
    Incapacitated {
        incapacitated_by: Option<Entity>,
    },
    Collision {
        is_new: bool,
        collision_with: Entity,
    },
}

/// This is used in JSON to specify which event types an effect should apply to
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerEventKind {
    Update,
    ReceiveDamage,
    GiveDamage,
    DamageBlocked,
    Incapacitated,
    Collision,
}

impl From<&PlayerEvent> for PlayerEventKind {
    fn from(params: &PlayerEvent) -> Self {
        use PlayerEvent::*;

        match params {
            Update { .. } => Self::Update,
            ReceiveDamage { .. } => Self::ReceiveDamage,
            GiveDamage { .. } => Self::GiveDamage,
            DamageBlocked { .. } => Self::DamageBlocked,
            Incapacitated { .. } => Self::Incapacitated,
            Collision { .. } => Self::Collision,
        }
    }
}

pub fn update_player_events(world: &mut World, delta_time: f32) -> Result<()> {
    let mut gave_damage = Vec::new();
    let mut function_calls = Vec::new();

    for (entity, (player, events)) in world.query::<(&mut Player, &mut PlayerEventQueue)>().iter() {
        events.queue.push(PlayerEvent::Update { delta_time });

        let mut i = 0;
        'events: while i < events.queue.len() {
            let event = events.queue.get(i).cloned().unwrap();

            if let PlayerEvent::ReceiveDamage {
                damage_from: _,
                is_from_left,
            } = event
            {
                let direction = if is_from_left == player.is_facing_left {
                    DamageDirection::Front
                } else {
                    DamageDirection::Back
                };

                'effects: for effect in &mut player.passive_effects {
                    if let Some(block) = &effect.damage_block {
                        if let Some(block_dir) = block.direction {
                            if direction == block_dir {
                                continue 'effects;
                            }
                        }

                        match block.kind {
                            PassiveEffectDamageBlockKind::IncrementUses => {
                                effect.use_cnt += 1;
                            }
                            PassiveEffectDamageBlockKind::EndEffect => {
                                effect.should_end = true;
                            }
                            _ => {}
                        }

                        println!("blocked");

                        events
                            .queue
                            .insert(i, PlayerEvent::DamageBlocked { is_from_left });

                        continue 'events;
                    }
                }
            }

            i += 1;
        }

        while let Some(event) = events.queue.pop() {
            match event {
                PlayerEvent::Update { delta_time } => {}
                PlayerEvent::ReceiveDamage {
                    damage_from,
                    is_from_left,
                } => {
                    let direction = if is_from_left == player.is_facing_left {
                        DamageDirection::Front
                    } else {
                        DamageDirection::Back
                    };

                    player.state = PlayerState::Dead;
                    player.damage_from = Some(direction);

                    if let Some(damage_from) = damage_from {
                        gave_damage.push((damage_from, entity));
                    }
                }
                PlayerEvent::GiveDamage { damage_to: _ } => {}
                PlayerEvent::DamageBlocked { is_from_left: _ } => {}
                PlayerEvent::Incapacitated {
                    incapacitated_by: _,
                } => {}
                PlayerEvent::Collision {
                    collision_with,
                    is_new,
                } => {}
            }

            let kind = PlayerEventKind::from(&event);

            for effect in &mut player.passive_effects {
                if !effect.should_begin && !effect.should_remove {
                    if let Some(funcs) = effect.on_event_fn.get(&kind) {
                        effect.use_cnt += 1;

                        if let Some(item_entity) = effect.item {
                            let mut item = world.get_mut::<Item>(item_entity).unwrap();

                            item.use_cnt += 1;
                        }

                        for f in funcs {
                            function_calls.push((*f, entity, effect.item, Some(event.clone())));
                        }
                    }
                }
            }
        }
    }

    for (f, player_entity, item_entity, event) in function_calls {
        f(world, player_entity, item_entity, event);
    }

    for (entity, target) in gave_damage {
        let mut events = world.get_mut::<PlayerEventQueue>(entity).unwrap();
        events.queue.push(PlayerEvent::GiveDamage {
            damage_to: Some(target),
        });
    }

    Ok(())
}
