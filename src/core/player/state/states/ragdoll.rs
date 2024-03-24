use rapier::vector;
use rapier2d::prelude as rapier;

use self::collisions::build_actor_rapier_collider;

use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::ragdoll"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

/// Track state of ragdoll.
#[derive(HasSchema, Clone, Default)]
pub struct PlayerRagdollState {
    /// If ticking, cannot twitch
    twitch_timer: Timer,

    // Last frame that started twitch animation
    last_twitch_anim_frame: u64,
}

pub fn handle_player_state(
    entities: Res<Entities>,
    time: Res<Time>,
    mut player_states: CompMut<PlayerState>,
    player_indexes: Comp<PlayerIdx>,
    mut transform: CompMut<Transform>,
    assets: Res<AssetServer>,
    player_inputs: Res<MatchInputs>,
    atlas_sprites: Comp<AtlasSprite>,
    mut animations: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut dynamic_bodies: CompMut<DynamicBody>,
    mut ragdoll_states: CompMut<PlayerRagdollState>,
    game_meta: Root<GameMeta>,
    mut collision_world: CollisionWorld,
    mut commands: Commands,
) {
    for (player_ent, (state, transform, animation, player_idx, atlas_sprite)) in
        entities.iter_with((
            &mut player_states,
            &mut transform,
            &mut animations,
            &player_indexes,
            &atlas_sprites,
        ))
    {
        if state.current != *ID {
            continue;
        };

        let meta_handle = player_inputs.players[player_idx.0 as usize].selected_player;
        let meta = &*assets.get(meta_handle);
        let player_physics = &game_meta.core.physics.player;
        let control = player_inputs.players[player_idx.0 as usize].control;

        match state.age {
            0 => {
                // TODO find right animation
                animation.current = "ragdoll".into();

                // drop item
                commands.add(PlayerCommand::set_inventory(player_ent, None));

                // Knock the player's hat off if they had one.
                commands.add(PlayerCommand::drop_hat(player_ent));

                // Set to simulate physics
                let dynamic_body =
                    dynamic_bodies.get_mut_or_insert(player_ent, DynamicBody::default);
                dynamic_body.is_dynamic = true;

                let body = bodies.get_mut(player_ent).unwrap();
                let player_physics = &game_meta.core.physics.player;
                let pop_vel = Vec2::new(0.0, player_physics.ragdoll_initial_pop);
                let ang_vel = player_physics.ragdoll_initial_ang_vel;
                let sprite_dir = atlas_sprite.flip_x;
                let current_vel = body.velocity;
                let additional_mass = player_physics.ragdoll_additional_mass;

                dynamic_body.push_simulation_command(Box::new(
                    move |body: &mut rapier::RigidBody| {
                        // Transfer current kinematic velocity to dynamic
                        body.set_linvel((current_vel).into(), true);

                        // Apply initial pop upward
                        body.apply_impulse((pop_vel * body.mass()).into(), true);

                        // Apply angular velocity
                        let angular_dir = if sprite_dir { 1.0 } else { -1.0 };
                        body.apply_torque_impulse(
                            ang_vel
                                * angular_dir
                                * body.mass_properties().effective_angular_inertia(),
                            true,
                        );

                        body.set_additional_mass(additional_mass, true);
                    },
                ));

                ragdoll_states.insert(
                    player_ent,
                    PlayerRagdollState {
                        twitch_timer: Timer::from_seconds(
                            player_physics.ragdoll_twitch_delay,
                            TimerMode::Once,
                        ),
                        last_twitch_anim_frame: 0,
                    },
                );
                use_ragdoll_collider(player_ent, meta, &mut collision_world)
            }
            n if n > 1 => {
                let ragdoll_state = ragdoll_states.get_mut(player_ent).unwrap();
                if control.ragdoll_just_pressed {
                    state.current = *idle::ID;
                    animation.current = ustr("idle");

                    // Switch back to kinematic
                    dynamic_bodies.get_mut(player_ent).unwrap().is_dynamic = false;
                    transform.rotation = Quat::default();

                    let pop_vel = Vec2::new(0.0, player_physics.ragdoll_initial_pop);
                    let body = bodies.get_mut(player_ent).unwrap();
                    body.velocity = pop_vel;

                    restore_player_collider(player_ent, meta, &mut collision_world);
                }
                // Handle twitching on input
                else if control.move_direction != Vec2::ZERO {
                    // Update timer determining if we may twitch on input
                    let timer = &mut ragdoll_state.twitch_timer;
                    timer.tick(time.delta());
                    let can_twitch = timer.finished();

                    animation.current = "ragdoll_twitch".into();
                    ragdoll_state.last_twitch_anim_frame = n;

                    // Check if touching solid, no twitching in air
                    // TODO: Maybe it's fun to allow the minor air control?
                    // let touching_solid = collision_world.tile_collision(*transform, body.shape)
                    //     != TileCollisionKind::Empty;

                    // Don't twitch when holding, only tap.
                    if can_twitch && control.just_moved
                    /*&& touching_solid*/
                    {
                        let dynamic_body = dynamic_bodies.get_mut(player_ent).unwrap();
                        let twitch_vel = player_physics.ragdoll_twitch_vel;

                        dynamic_body.push_simulation_command(Box::new(
                            move |body: &mut rapier::RigidBody| {
                                let mut lin_vel = *body.linvel();
                                let dir_x = control.move_direction.x;
                                lin_vel += vector!(dir_x * twitch_vel, 3.0 * twitch_vel);
                                body.set_linvel(lin_vel, true);
                            },
                        ));

                        timer.reset();
                    }
                } else {
                    // Not twitching, back to normal animation if twitch anim held for at least 2 frames
                    if n >= ragdoll_state.last_twitch_anim_frame + 4 {
                        animation.current = "ragdoll".into();
                    }
                }
            }
            _ => {}
        }
    }
}

/// Switch entity to a capsule collider based on collider
/// from [`PlayerMeta`].
pub fn use_ragdoll_collider(
    entity: Entity,
    player_meta: &PlayerMeta,
    collision_world: &mut CollisionWorld,
) {
    // Build capsule from player size.
    let radius = player_meta.body_size.x / 2.0;
    let half_length = player_meta.body_size.y * 0.5 - radius;
    let capsule = ColliderShape::CapsuleY {
        half_length,
        radius,
    };
    let mut builder = build_actor_rapier_collider(entity, capsule.shared_shape());

    // Shift capsule slightly to lineup with animation
    let translation = rapier::Translation::new(0.0, 5.0);
    builder = builder.position(translation.into());

    collision_world.set_actor_shape_from_builder(entity, builder, capsule);
}

/// Restore player shape back to default from `PlayerMeta`.
fn restore_player_collider(
    player_ent: Entity,
    player_meta: &PlayerMeta,
    collision_world: &mut CollisionWorld,
) {
    collision_world.set_actor_shape(player_ent, player_collider_shape(player_meta));
}
