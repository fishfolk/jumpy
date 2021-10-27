use macroquad::{
    experimental::{
        collections::storage,
        scene::{Handle, HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use macroquad_platformer::Tile;

use serde::{Deserialize, Serialize};

use crate::json::OneOrMany;
use crate::{
    capabilities::NetworkReplicate,
    components::{AnimationParams, AnimationPlayer, PhysicsBody},
    json, GameWorld, Player,
};

use super::{active_effect_coroutine, ActiveEffectParams, AnyEffectParams};

/// This contains commonly used groups of triggers
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggeredEffectTriggerGroup {
    All,
    AllPlayers,
    AllExceptPlayer,
    AllExceptGround,
}

impl From<TriggeredEffectTriggerGroup> for Vec<TriggeredEffectTrigger> {
    fn from(group: TriggeredEffectTriggerGroup) -> Self {
        match group {
            TriggeredEffectTriggerGroup::All => vec![
                TriggeredEffectTrigger::Player,
                TriggeredEffectTrigger::Enemy,
                TriggeredEffectTrigger::Ground,
                TriggeredEffectTrigger::Explosion,
                TriggeredEffectTrigger::Projectile,
            ],
            TriggeredEffectTriggerGroup::AllPlayers => vec![
                TriggeredEffectTrigger::Player,
                TriggeredEffectTrigger::Enemy,
            ],
            TriggeredEffectTriggerGroup::AllExceptPlayer => vec![
                TriggeredEffectTrigger::Enemy,
                TriggeredEffectTrigger::Ground,
                TriggeredEffectTrigger::Explosion,
                TriggeredEffectTrigger::Projectile,
            ],
            TriggeredEffectTriggerGroup::AllExceptGround => vec![
                TriggeredEffectTrigger::Player,
                TriggeredEffectTrigger::Enemy,
                TriggeredEffectTrigger::Explosion,
                TriggeredEffectTrigger::Projectile,
            ],
        }
    }
}

/// The various collision types that can trigger a `TriggeredEffect`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggeredEffectTrigger {
    /// The player that deployed the effect
    Player,
    /// Enemy players
    Enemy,
    /// Ground tiles (all tiles with collision, except platforms)
    Ground,
    /// Explosion effects
    Explosion,
    /// Projectile hit
    Projectile,
}

/// This is an untagged enum that makes it possible to accept a single `TriggeredEffectTrigger`
/// variant (`Single` variant), a vector of `TriggeredEffectTrigger` (`Vec` variant) or a variant
/// of `TriggeredEffectTriggerGroup` (`Group` variant), when deserializing JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TriggeredEffectTriggerParams {
    Single(TriggeredEffectTrigger),
    Vec(Vec<TriggeredEffectTrigger>),
    Group(TriggeredEffectTriggerGroup),
}

impl From<TriggeredEffectTriggerParams> for Vec<TriggeredEffectTrigger> {
    fn from(params: TriggeredEffectTriggerParams) -> Self {
        match params {
            TriggeredEffectTriggerParams::Single(trigger) => vec![trigger],
            TriggeredEffectTriggerParams::Vec(triggers) => triggers,
            TriggeredEffectTriggerParams::Group(group) => group.into(),
        }
    }
}

impl Default for TriggeredEffectTriggerParams {
    fn default() -> Self {
        Self::Single(TriggeredEffectTrigger::Player)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffectParams {
    /// The effects to instantiate when the triggers condition is met. Can be either a single
    /// effect or a vec of effects, either passive or active
    pub effects: OneOrMany<AnyEffectParams>,
    /// This specifies the size of the trigger.
    #[serde(with = "json::vec2_def")]
    pub size: Vec2,
    /// This specifies the valid trigger conditions for the trigger.
    #[serde(default = "TriggeredEffectTriggerParams::default")]
    pub trigger: TriggeredEffectTriggerParams,
    /// This specifies the velocity of the triggers body, when it is instantiated.
    #[serde(default, with = "json::vec2_def")]
    pub velocity: Vec2,
    /// This can be used to add an animated sprite to the trigger. If only a sprite is desired, an
    /// animation with only one frame can be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animation: Option<AnimationParams>,
    /// This specifies the delay between the the trigger is instantiated and when it will be
    /// possible to trigger it.
    ///
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default)]
    pub activation_delay: f32,
    /// This specifies the delay between the triggers conditions are met and the effect is triggered.
    ///
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default)]
    pub trigger_delay: f32,
    /// If a value is specified the effect will trigger automatically after `value` time has passed.
    ///
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timed_trigger: Option<f32>,
    /// If this is `true` the trigger is kicked by a player, if it hits him while he is facing it
    #[serde(default)]
    pub is_kickable: bool,
}

impl Default for TriggeredEffectParams {
    fn default() -> Self {
        TriggeredEffectParams {
            effects: OneOrMany::Many(Vec::new()),
            size: Vec2::ONE,
            trigger: TriggeredEffectTriggerParams::Vec(Vec::new()),
            velocity: Vec2::ZERO,
            animation: None,
            activation_delay: 0.0,
            trigger_delay: 0.0,
            timed_trigger: None,
            is_kickable: false,
        }
    }
}

struct TriggeredEffect {
    pub owner: Handle<Player>,
    pub size: Vec2,
    pub trigger: Vec<TriggeredEffectTrigger>,
    pub effects: Vec<AnyEffectParams>,
    pub animation_player: Option<AnimationPlayer>,
    pub body: PhysicsBody,
    pub activation_delay: f32,
    pub trigger_delay: f32,
    pub timed_trigger: Option<f32>,
    pub is_kickable: bool,
    pub is_triggered: bool,
    /// This can be used to trigger the effect immediately, ignoring delay timers.
    /// Also requires `is_triggered` to be set to `true`, for this to work.
    pub should_override_delay: bool,
    /// This holds a handle to the player that triggered the effect, if applicable.
    triggered_by: Option<Handle<Player>>,
    kick_delay_timer: f32,
    activation_timer: f32,
    trigger_delay_timer: f32,
    timed_trigger_timer: f32,
}

impl TriggeredEffect {
    fn apply_trigger(&mut self, trigger: TriggeredEffectTrigger, triggered_by: Option<Handle<Player>>) {
        self.is_triggered = true;

        if trigger == TriggeredEffectTrigger::Explosion
            || trigger == TriggeredEffectTrigger::Projectile
        {
            self.should_override_delay = true;
        }

        self.triggered_by = triggered_by;
    }
}

#[derive(Default)]
pub struct TriggeredEffects {
    active: Vec<TriggeredEffect>,
}

impl TriggeredEffects {
    const KICK_FORCE: f32 = 800.0;

    // Delay before the player that deploy a kickable effect can kick it (to avoid insta-kicking it)
    const KICK_DELAY: f32 = 0.22;

    pub fn new() -> Self {
        TriggeredEffects { active: Vec::new() }
    }

    pub fn spawn(&mut self, owner: Handle<Player>, position: Vec2, params: TriggeredEffectParams) {
        let trigger = params.trigger.into();

        let mut animation_player = None;
        if let Some(animation_params) = params.animation {
            animation_player = Some(AnimationPlayer::new(animation_params));
        }

        let mut body = {
            let mut game_world = storage::get_mut::<GameWorld>();
            PhysicsBody::new(
                &mut game_world.collision_world,
                position,
                0.0,
                params.size,
                false,
                true,
                None,
            )
        };

        body.velocity = params.velocity;

        self.active.push(TriggeredEffect {
            owner,
            size: params.size,
            trigger,
            effects: params.effects.into(),
            animation_player,
            body,
            activation_delay: params.activation_delay,
            activation_timer: 0.0,
            trigger_delay: params.trigger_delay,
            trigger_delay_timer: 0.0,
            timed_trigger: params.timed_trigger,
            timed_trigger_timer: 0.0,
            is_kickable: params.is_kickable,
            kick_delay_timer: 0.0,
            is_triggered: false,
            should_override_delay: false,
            triggered_by: None,
        })
    }

    #[allow(dead_code)]
    pub fn check_triggers(&mut self, trigger: TriggeredEffectTrigger, collider: &Rect, triggered_by: Option<Handle<Player>>) {
        for effect in &mut self.active {
            if collider.overlaps(&effect.body.get_collider_rect())
                && effect.trigger.contains(&trigger)
            {
                effect.apply_trigger(trigger, triggered_by);
                continue;
            }
        }
    }

    pub fn check_triggers_circle(&mut self, trigger: TriggeredEffectTrigger, collider: &Circle, triggered_by: Option<Handle<Player>>) {
        for effect in &mut self.active {
            if collider.overlaps_rect(&effect.body.get_collider_rect())
                && effect.trigger.contains(&trigger)
            {
                effect.apply_trigger(trigger, triggered_by);
                continue;
            }
        }
    }

    pub fn check_triggers_point(&mut self, trigger: TriggeredEffectTrigger, point: Vec2, triggered_by: Option<Handle<Player>>) {
        for effect in &mut self.active {
            if effect.body.get_collider_rect().contains(point) && effect.trigger.contains(&trigger)
            {
                effect.apply_trigger(trigger, triggered_by);
                continue;
            }
        }
    }

    fn network_update(mut node: RefMut<Self>) {
        let mut i = 0;
        while i < node.active.len() {
            let trigger = &mut node.active[i];

            trigger.body.update();

            let dt = get_frame_time();

            if let Some(timed_trigger) = trigger.timed_trigger {
                trigger.timed_trigger_timer += dt;
                if trigger.timed_trigger_timer >= timed_trigger {
                    trigger.is_triggered = true;
                }
            }

            if trigger.kick_delay_timer < Self::KICK_DELAY {
                trigger.kick_delay_timer += dt;
            }

            if trigger.activation_delay > 0.0 {
                trigger.activation_timer += dt;
            }

            if trigger.is_triggered {
                trigger.trigger_delay_timer += dt;
            }

            if !trigger.is_triggered && trigger.activation_timer >= trigger.activation_delay {
                let collider = Rect::new(
                    trigger.body.position.x,
                    trigger.body.position.y,
                    trigger.size.x,
                    trigger.size.y,
                );

                let can_be_triggered_by_player =
                    trigger.trigger.contains(&TriggeredEffectTrigger::Player);
                let can_be_triggered_by_enemy =
                    trigger.trigger.contains(&TriggeredEffectTrigger::Enemy);
                let can_be_triggered_by_ground =
                    trigger.trigger.contains(&TriggeredEffectTrigger::Ground);

                if can_be_triggered_by_player || can_be_triggered_by_enemy {
                    let mut _player = None;
                    if (trigger.is_kickable && trigger.kick_delay_timer < Self::KICK_DELAY)
                        || (!can_be_triggered_by_player && !trigger.is_kickable)
                    {
                        _player = scene::try_get_node(trigger.owner)
                    }

                    for player in scene::find_nodes_by_type::<Player>() {
                        if collider.overlaps(&player.get_collider_rect()) {
                            if trigger.is_kickable {
                                if !player.body.is_facing_right
                                    && trigger.body.position.x
                                        < player.body.position.x + player.body.size.x
                                {
                                    trigger.body.velocity.x = -Self::KICK_FORCE;
                                } else if player.body.is_facing_right
                                    && trigger.body.position.x > player.body.position.x
                                {
                                    trigger.body.velocity.x = Self::KICK_FORCE;
                                } else {
                                    trigger.is_triggered = true;
                                    trigger.triggered_by = Some(player.handle());
                                }
                            } else {
                                trigger.is_triggered = true;
                                trigger.triggered_by = Some(player.handle());
                            }

                            break;
                        }
                    }
                }

                if !trigger.is_triggered && can_be_triggered_by_ground {
                    if trigger.body.is_on_ground {
                        trigger.is_triggered = true;
                    } else {
                        let game_world = storage::get::<GameWorld>();
                        let tile = game_world.collision_world.collide_solids(
                            collider.point(),
                            collider.w as i32,
                            collider.h as i32,
                        );

                        if tile == Tile::Solid {
                            trigger.is_triggered = true;
                        }
                    }
                }
            }

            if trigger.is_triggered
                && (trigger.should_override_delay
                    || trigger.trigger_delay_timer >= trigger.trigger_delay)
            {
                for params in trigger.effects.drain(0..) {
                    match params {
                        AnyEffectParams::Active(params) => {
                            active_effect_coroutine(trigger.owner, trigger.body.position, params);
                        }
                        AnyEffectParams::Passive(params) => {
                            if let Some(triggered_by) = trigger.triggered_by {
                                if let Some(mut player) = scene::try_get_node(triggered_by) {
                                    player.add_passive_effect(params);
                                }
                            }
                        }
                    }
                }

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
                animation_player.draw(trigger.body.position, 0.0, false, false);
            }
        }
    }
}
