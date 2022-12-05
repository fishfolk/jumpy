//! The crate item.
//!
//! This module is inconsistently named with the rest of the modules ( i.e. has an `_item` suffix )
//! because `crate` is a Rust keyword.

use std::cmp::Ordering;

use crate::{physics::collisions::Rect, player::PlayerKillCommand};

use super::*;

pub struct StompBootsPlugin;

impl Plugin for StompBootsPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_idle_stomp_boots)
            .add_rollback_system(RollbackStage::Update, kill_stomped_players);
    }
}

/// Marker component added to things ( presumably players, but not necessarily! ) that are wearing
/// stomp boots
#[derive(Debug, Clone, Copy, Default, Component)]
pub struct WearingStompBoots;

#[derive(Debug, Clone, Copy, Component)]
pub struct IdleStompBoots {
    pub spawner: Entity,
}

fn pre_update_in_game(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    // Hydrate any newly-spawned crates
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::StompBoots {
            body_size,
            body_offset,
            map_icon_handle,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Item {
                    script: "core:stomp_boots".into(),
                })
                .insert(EntityName("Item: Stomp Boots".into()))
                .insert(IdleStompBoots { spawner: entity })
                .insert(AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: map_icon_handle.clone_weak(),
                    repeat: false,
                    ..default()
                })
                .insert(map_element_handle.clone_weak())
                .insert_bundle(VisibilityBundle::default())
                .insert_bundle(TransformBundle {
                    local: *transform,
                    ..default()
                })
                .insert(KinematicBody {
                    size: *body_size,
                    offset: *body_offset,
                    gravity: 1.0,
                    has_mass: true,
                    has_friction: true,
                    ..default()
                });
        }
    }
}

fn update_idle_stomp_boots(
    mut commands: Commands,
    mut players: Query<(&mut AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &IdleStompBoots,
            &mut Transform,
            &mut AnimatedSprite,
            &mut KinematicBody,
            &Handle<MapElementMeta>,
            Option<&Parent>,
            Option<&ItemUsed>,
            Option<&ItemDropped>,
        ),
        Without<PlayerIdx>,
    >,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    let mut items = grenades.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (
        _,
        item_ent,
        boots,
        mut transform,
        mut sprite,
        mut body,
        meta_handle,
        parent,
        used,
        dropped,
    ) in items
    {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::StompBoots {
            grab_offset,
            player_decoration_handle,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(parent) = parent {
            let (mut player_sprite, ..) =
                players.get_mut(parent.get()).expect("Parent is not player");

            // Deactivate items while held
            body.is_deactivated = true;

            // Flip the sprite to match the player orientation
            let flip = player_sprite.flip_x;
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            transform.translation.x = grab_offset.x * flip_factor;
            transform.translation.y = grab_offset.y;
            transform.translation.z = 1.0;

            // If the item is being used
            if used.is_some() {
                // Use up the boots
                commands.entity(item_ent).despawn();
                // This will make the boots respawn at their spawn point
                commands
                    .entity(boots.spawner)
                    .remove::<MapElementHydrated>();
                commands.entity(parent.get()).insert(WearingStompBoots);

                // Have the player wear the boots
                if !player_sprite
                    .stacked_atlases
                    .contains(player_decoration_handle)
                {
                    player_sprite
                        .stacked_atlases
                        .push(player_decoration_handle.clone_weak());
                }
            }
        }

        // If the item is dropped
        if let Some(dropped) = dropped {
            commands.entity(item_ent).remove::<ItemDropped>();
            let (player_sprite, player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physics
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.start = 0;
            sprite.end = 0;
            body.velocity = player_body.velocity;
            body.is_spawning = true;

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };

            // Drop item at player position
            transform.translation =
                player_transform.translation + (*grab_offset * horizontal_flip_factor).extend(1.0);
        }
    }
}

fn kill_stomped_players(
    mut commands: Commands,
    players: Query<Entity, With<PlayerIdx>>,
    stompers: Query<(Entity, &KinematicBody), With<WearingStompBoots>>,
    collision_world: CollisionWorld,
) {
    // For all players wearing stomp boots
    for (stomper, stomper_body) in &stompers {
        let collisions = collision_world.actor_collisions(stomper);

        // Require that the stomper be moving down to stomp
        if stomper_body.velocity.y.partial_cmp(&0.0) != Some(Ordering::Less) {
            continue;
        }

        // For every collision
        for colliding_ent in collisions {
            // If that collision is with a player
            if players.contains(colliding_ent) {
                let mut stomper_rect = collision_world.get_collider(stomper).rect();
                let mut player_rect = collision_world.get_collider(colliding_ent).rect();

                // Modify the stomper rect to represent the feet of the stomper, and modify the
                // player rect to represent the head of the player.
                //
                // TODO: We may want better stomp logic than
                stomper_rect = Rect::new(
                    stomper_rect.min().x,
                    stomper_rect.min().y,
                    stomper_rect.width(),
                    stomper_rect.height() / 2.0,
                );

                let player_height = player_rect.height() / 5.0;
                player_rect = Rect::new(
                    player_rect.min().x,
                    player_rect.max().y - player_height,
                    player_rect.width(),
                    player_height,
                );

                if stomper_rect.overlaps(&player_rect) {
                    commands.add(PlayerKillCommand::new(colliding_ent));
                }
            }
        }
    }
}
