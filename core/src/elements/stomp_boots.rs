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
pub struct StompBoots;

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
    mut item_throws: CompMut<ItemThrow>,
    mut item_grabs: CompMut<ItemGrab>,
    mut respawn_points: CompMut<DehydrateOutOfBounds>,
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
            grab_offset,
            body_size,
            map_icon,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            item_throws.insert(entity, ItemThrow::strength(0.0));
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: key!("grab_2"),
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            stomp_boots.insert(entity, StompBoots);
            atlas_sprites.insert(entity, AtlasSprite::new(map_icon.clone()));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
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
    mut stomp_boots: CompMut<StompBoots>,
    items_used: Comp<ItemUsed>,
    player_inventories: PlayerInventories,
    mut inventoris: CompMut<Inventory>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut commands: Commands,
    spawners: Comp<DehydrateOutOfBounds>,
) {
    for (entity, (_stomp_boots, element_handle, spawner)) in
        entities.iter_with((&mut stomp_boots, &element_handles, &spawners))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::StompBoots {
            player_decoration,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(Inv { player, .. }) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            // If the item is being used
            let is_item_used = items_used.get(entity).is_some();
            let player_decoration = player_decoration.clone();

            if is_item_used {
                hydrated.remove(**spawner);
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
    }
}

fn update_wearer(
    entities: Res<Entities>,
    mut commands: Commands,
    wearing_stomp_boots: Comp<WearingStompBoots>,
    player_indexes: Comp<PlayerIdx>,
    collision_world: CollisionWorld,
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
                if kinematic_body.bounding_box(*wearer_transform).bottom()
                    > player_kinematic_body
                        .bounding_box(*player_transform)
                        .center()
                        .y
                {
                    commands.add(PlayerCommand::kill(
                        player,
                        Some(player_transform.translation.xy()),
                    ))
                }
            });
    }
}
