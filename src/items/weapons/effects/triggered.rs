use macroquad::{
    experimental::{
        collections::storage,
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use macroquad_platformer::Tile;

use crate::{
    capabilities::NetworkReplicate,
    components::{AnimationParams, AnimationPlayer, PhysicsBody},
    GameWorld, Player,
};

use super::{weapon_effect_coroutine, WeaponEffectParams, WeaponEffectTriggerKind};

pub struct TriggeredEffectParams {
    pub offset: Vec2,
    pub velocity: Vec2,
    pub animation: Option<AnimationParams>,
    pub is_friendly_fire: bool,
    pub activation_delay: f32,
    pub timed_trigger: Option<f32>,
}

impl Default for TriggeredEffectParams {
    fn default() -> Self {
        TriggeredEffectParams {
            offset: Vec2::ZERO,
            velocity: Vec2::ZERO,
            animation: None,
            is_friendly_fire: false,
            activation_delay: 0.0,
            timed_trigger: None,
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
    pub offset: Vec2,
    pub activation_delay: f32,
    pub activation_timer: f32,
    pub timed_trigger: Option<f32>,
    pub timed_trigger_timer: f32,
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

        let mut body = {
            let mut game_world = storage::get_mut::<GameWorld>();
            PhysicsBody::new(
                &mut game_world.collision_world,
                position + params.offset,
                0.0,
                size,
                false,
                true,
            )
        };

        body.velocity = params.velocity;

        self.active.push(TriggeredEffect {
            owner,
            size,
            kind,
            effect,
            animation_player,
            body,
            offset: params.offset,
            is_friendly_fire: params.is_friendly_fire,
            activation_delay: params.activation_delay,
            activation_timer: 0.0,
            timed_trigger: params.timed_trigger,
            timed_trigger_timer: 0.0,
        })
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        while i < node.active.len() {
            let trigger = &mut node.active[i];

            trigger.body.update();

            let mut is_triggered = false;

            if let Some(timed_trigger) = trigger.timed_trigger {
                trigger.timed_trigger_timer += get_frame_time();
                if trigger.timed_trigger_timer >= timed_trigger {
                    is_triggered = true;
                }
            }

            if trigger.activation_delay > 0.0 {
                trigger.activation_timer += get_frame_time();
            }

            if !is_triggered && trigger.activation_timer >= trigger.activation_delay {
                let collider = Rect::new(
                    trigger.body.pos.x + trigger.offset.x,
                    trigger.body.pos.y + trigger.offset.y,
                    trigger.size.x,
                    trigger.size.y,
                );

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
                    if trigger.body.is_on_ground {
                        is_triggered = true;
                    } else {
                        let game_world = storage::get::<GameWorld>();
                        let tile = game_world.collision_world.collide_solids(
                            collider.point(),
                            collider.w as i32,
                            collider.h as i32,
                        );
                        if tile == Tile::Solid {
                            is_triggered = true;
                        }
                    }
                }
            }

            if is_triggered {
                weapon_effect_coroutine(trigger.owner, trigger.body.pos, trigger.effect.clone());
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

    fn update(mut node: RefMut<Self>) {
        for trigger in &mut node.active {
            if let Some(animation_player) = trigger.animation_player.as_mut() {
                animation_player.update();
            }
        }
    }

    fn draw(mut node: RefMut<Self>) {
        for trigger in &mut node.active {
            if let Some(animation_player) = &trigger.animation_player {
                animation_player.draw(trigger.body.pos, 0.0, false, false);
            }
        }
    }
}
