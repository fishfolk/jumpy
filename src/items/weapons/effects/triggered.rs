use macroquad::{
    experimental::{
        collections::storage,
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use crate::capabilities::NetworkReplicate;
use crate::components::{AnimationParams, AnimationPlayer, PhysicsBody};

use super::{WeaponEffectParams, WeaponEffectTriggerKind};

use crate::items::weapons::weapon_effect_coroutine;
use crate::{GameWorld, Player};

pub struct TriggeredEffectParams {
    pub is_friendly_fire: bool,
    pub activation_delay: f32,
    pub animation: Option<AnimationParams>,
}

impl Default for TriggeredEffectParams {
    fn default() -> Self {
        TriggeredEffectParams {
            is_friendly_fire: false,
            activation_delay: 0.0,
            animation: None,
        }
    }
}

struct TriggeredEffect {
    pub owner: Handle<Player>,
    pub size: Vec2,
    pub kind: WeaponEffectTriggerKind,
    pub effect: WeaponEffectParams,
    pub is_friendly_fire: bool,
    pub animation_player: Option<AnimationPlayer>,
    pub body: PhysicsBody,
    pub activation_delay: f32,
    pub activation_timer: f32,
}

pub struct TriggeredEffects {
    active: Vec<TriggeredEffect>,
}

impl TriggeredEffects {
    pub fn new() -> Self {
        TriggeredEffects { active: Vec::new() }
    }

    pub fn spawn(
        &mut self,
        owner: Handle<Player>,
        kind: WeaponEffectTriggerKind,
        position: Vec2,
        size: Vec2,
        effect: WeaponEffectParams,
        params: TriggeredEffectParams,
    ) {
        let mut animation_player = None;

        if let Some(animation_params) = params.animation {
            animation_player = Some(AnimationPlayer::new(animation_params));
        }

        let body = {
            let mut game_world = storage::get_mut::<GameWorld>();
            PhysicsBody::new(&mut game_world.collision_world, position, 0.0, size, false)
        };

        self.active.push(TriggeredEffect {
            owner,
            size,
            kind,
            effect,
            animation_player,
            body,
            is_friendly_fire: params.is_friendly_fire,
            activation_delay: params.activation_delay,
            activation_timer: 0.0,
        })
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        while i < node.active.len() {
            let trigger = &mut node.active[i];

            trigger.body.update();

            if trigger.activation_timer >= trigger.activation_delay {
                let collider = Rect::new(
                    trigger.body.pos.x,
                    trigger.body.pos.y,
                    trigger.size.x,
                    trigger.size.y,
                );

                let mut is_triggered = false;

                if trigger.kind == WeaponEffectTriggerKind::Player
                    || trigger.kind == WeaponEffectTriggerKind::Both
                {
                    let _player = if trigger.is_friendly_fire {
                        None
                    } else {
                        scene::try_get_node(trigger.owner)
                    };

                    for player in scene::find_nodes_by_type::<Player>() {
                        if collider.overlaps(&player.get_collider()) {
                            is_triggered = true;
                            break;
                        }
                    }
                }

                if !is_triggered
                    && (trigger.kind == WeaponEffectTriggerKind::Ground
                        || trigger.kind == WeaponEffectTriggerKind::Both)
                {
                    let game_world = storage::get::<GameWorld>();
                    if !game_world.map.get_collisions(&collider).is_empty() {
                        is_triggered = true;
                    }
                }

                if is_triggered {
                    weapon_effect_coroutine(
                        trigger.owner,
                        trigger.body.pos,
                        trigger.effect.clone(),
                    );
                    node.active.remove(i);
                    continue;
                }
            }

            i += 1;
        }
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle)
                .unwrap()
                .to_typed::<TriggeredEffects>();
            TriggeredEffects::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}

impl Node for TriggeredEffects {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }

    fn update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        for trigger in &mut node.active {
            if trigger.activation_delay > 0.0 {
                trigger.activation_timer += get_frame_time();
            }

            if let Some(animation_player) = trigger.animation_player.as_mut() {
                animation_player.update();
            }
        }
    }

    fn draw(node: RefMut<Self>)
    where
        Self: Sized,
    {
        for trigger in &node.active {
            if let Some(animation_player) = &trigger.animation_player {
                animation_player.draw(trigger.body.pos, 0.0, None, false, false);
            }
        }
    }
}
