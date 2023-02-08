use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update)
        .add_system_to_stage(CoreStage::PostUpdate, update_wearer);
}

/// Marker component added to things ( presumably players, but not necessarily! ) that are wearing
/// stomp boots
#[derive(Debug, Clone, Copy, Default, TypeUlid)]
#[ulid = "01GR0P6HDCXJA6P2VNN8E1TH6Q"]
pub struct WearingStompBoots;

#[derive(Copy, Clone, Debug, TypeUlid)]
#[ulid = "01GR0G9X4TME8E7NG0Z3DD26QW"]
pub struct StompBoots {
    spawner: Entity,
}

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut stomp_boots: CompMut<StompBoots>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut items: CompMut<Item>,
    mut respawn_points: CompMut<MapRespawnPoint>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawners = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawners {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::StompBoots {
            body_size,
            body_offset,
            map_icon,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            stomp_boots.insert(
                entity,
                StompBoots {
                    spawner: spawner_ent,
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(map_icon.clone()));
            respawn_points.insert(entity, MapRespawnPoint(transform.translation));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    size: *body_size,
                    offset: *body_offset,
                    has_mass: true,
                    has_friction: true,
                    gravity: game_meta.physics.gravity,
                    ..default()
                },
            );
        }
    }
}

fn update(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut transforms: CompMut<Transform>,
    mut stomp_boots: CompMut<StompBoots>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    items_used: Comp<ItemUsed>,
    mut items_dropped: CompMut<ItemDropped>,
    player_inventories: PlayerInventories,
    mut inventoris: CompMut<Inventory>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut commands: Commands,
) {
    for (entity, (stomp_boots, element_handle)) in
        entities.iter_with((&mut stomp_boots, &element_handles))
    {
        let spawner = stomp_boots.spawner;
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::StompBoots {
            grab_offset,
            player_decoration,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;
            let player_sprite = sprites.get_mut(player).unwrap();

            let body = bodies.get_mut(entity).unwrap();
            body.is_deactivated = true;

            let flip = player_sprite.flip_x;
            let sprite = sprites.get_mut(entity).unwrap();
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            let player_translation = transforms.get(player).unwrap().translation;
            let transform = transforms.get_mut(entity).unwrap();
            let offset = Vec3::new(grab_offset.x * flip_factor, grab_offset.y, 1.0);
            transform.translation = player_translation + offset;
            transform.rotation = Quat::IDENTITY;

            // If the item is being used
            let is_item_used = items_used.get(entity).is_some();
            let player_decoration = player_decoration.clone();

            if is_item_used {
                hydrated.remove(spawner);
                inventoris.insert(player, Inventory(None));
                commands.add(
                    move |mut entities: ResMut<Entities>,
                          mut sprites: CompMut<AtlasSprite>,
                          mut attachments: CompMut<Attachment>,
                          mut transforms: CompMut<Transform>,
                          mut wearing_stomp_boots: CompMut<WearingStompBoots>| {
                        entities.kill(entity);

                        let attachment_ent = entities.create();
                        let attachment = Attachment {
                            entity: player,
                            offset: Vec3::ZERO,
                            sync_animation: true,
                        };
                        attachments.insert(attachment_ent, attachment);
                        sprites.insert(attachment_ent, AtlasSprite::new(player_decoration.clone()));
                        transforms.insert(attachment_ent, Transform::default());
                        wearing_stomp_boots.insert(player, WearingStompBoots);
                    },
                );
            }
        }

        // If the item was dropped
        if let Some(dropped) = items_dropped.get(entity).copied() {
            let player = dropped.player;

            items_dropped.remove(entity);
            let player_translation = transforms.get(dropped.player).unwrap().translation;
            let player_velocity = bodies.get(player).unwrap().velocity;

            let body = bodies.get_mut(entity).unwrap();
            let sprite = sprites.get_mut(entity).unwrap();

            // Re-activate physics
            body.is_deactivated = false;

            let horizontal_flip_factor = if sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };
            body.velocity = horizontal_flip_factor + player_velocity;

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}

fn update_wearer(
    entities: Res<Entities>,
    wearing_stomp_boots: Comp<WearingStompBoots>,
    player_indexes: Comp<PlayerIdx>,
    collision_world: CollisionWorld,
    mut player_events: ResMut<PlayerEvents>,
    kinematic_bodies: Comp<KinematicBody>,
    transforms: Comp<Transform>,
) {
    for (entity, _) in entities.iter_with(&wearing_stomp_boots) {
        let kinematic_body = kinematic_bodies.get(entity).unwrap();
        if kinematic_body.velocity.y > 0.
            || kinematic_body.is_on_ground
            || kinematic_body.is_on_platform
        {
            continue;
        }
        collision_world
            .actor_collisions(entity)
            .into_iter()
            .filter(|&x| player_indexes.contains(x))
            .for_each(|player| {
                let wearer_transform = transforms
                    .get(entity)
                    .expect("stomp boots wearer should have Transform component");
                let player_transform = transforms.get(player).unwrap();
                let player_kinematic_body = kinematic_bodies.get(player).unwrap();
                if kinematic_body
                    .collider_rect(wearer_transform.translation)
                    .bottom()
                    > player_kinematic_body
                        .collider_rect(player_transform.translation)
                        .center()
                        .y
                {
                    player_events.kill(player)
                }
            });
    }
}
