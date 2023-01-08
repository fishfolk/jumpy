use crate::networking::RollbackIdWrapper;

use super::*;

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct IdleBlunderBussItem {
    /// The entity ID of the map element that spawned the crate
    spawner: Entity,
}

impl Default for IdleBlunderBussItem {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
        }
    }
}

pub struct BlunderBussPlugin;

impl Plugin for BlunderBussPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_idle_blunderbuss);
    }
}

fn pre_update_in_game(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::Blunderbuss {
            body_size,
            body_offset,
            atlas_handle,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Item {
                    script: "core:blunderbass".into(),
                })
                .insert(Name::new("Item: Blunderbass"))
                .insert(IdleBlunderBussItem { spawner: entity })
                .insert(AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: atlas_handle.inner.clone(),
                    repeat: false,
                    ..default()
                })
                .insert(map_element_handle.clone_weak())
                .insert_bundle(VisibilityBundle::default())
                .insert(MapRespawnPoint(transform.translation))
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

fn update_idle_blunderbuss(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut blunderbusses: Query<
        (
            &Rollback,
            Entity,
            &IdleBlunderBussItem,
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
    mut ridp: ResMut<RollbackIdWrapper>,
    element_assets: ResMut<Assets<MapElementMeta>>,
    effects: Res<AudioChannel<EffectsChannel>>,
) {
    let mut items = blunderbusses.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (
        _,
        item_ent,
        blunderbuss,
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
        let BuiltinElementKind::Blunderbuss {
            grab_offset,
            atlas_handle,
            bullet_sound_handle,
            bullet_velocity,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(parent) = parent {
            let (player_sprite, player_transform, player_body, ..) =
                players.get(parent.get()).expect("Parent is not player");

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
                // Spawn a bullet
            }
        }

        // If the item was dropped
        if let Some(dropped) = dropped {
            println!("Dropped");

            commands.entity(item_ent).remove::<ItemDropped>();
            let (.., player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physicsc
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.index = 0;
            sprite.start = 0;
            sprite.end = 0;
            body.velocity = player_body.velocity;
            body.is_spawning = true;

            // Drop item at middle of player
            transform.translation.y = player_transform.translation.y - 30.0;
            transform.translation.x = player_transform.translation.x;
            transform.translation.z = player_transform.translation.z;
        }
    }
}
