use crate::{physics::collisions::TileCollisionKind, prelude::*, player};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate_crates)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_crates)
        .add_system_to_stage(CoreStage::PostUpdate, update_thrown_crates);
}

#[derive(Clone, TypeUlid)]
#[ulid = "01GREP3MZXY4A14PQ8GRKS0RVY"]
struct IdleCrate;

#[derive(Clone, TypeUlid)]
#[ulid = "01GREP80RJSH9T9MWC88CG2G03"]
struct ThrownCrate {
    owner: Entity,
    age: f32,
    crate_break_state: u8,
    was_colliding: bool,
}

fn hydrate_crates(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut idle_crates: CompMut<IdleCrate>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
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

    for spawner in spawners {
        let transform = *transforms.get(spawner).unwrap();
        let element_handle = element_handles.get(spawner).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else{
            continue;
        };

        let BuiltinElementKind::Crate{
            atlas,
            fin_anim,
            grab_offset,
            body_size,
            throw_velocity,
            bounciness,
            ..
        } = &element_meta.builtin else{
            continue;
        };

        hydrated.insert(spawner, MapElementHydrated);

        let entity = entities.create();
        items.insert(entity, Item);
        idle_crates.insert(entity, IdleCrate);
        item_throws.insert(entity, ItemThrow::strength(*throw_velocity));
        item_grabs.insert(
            entity,
            ItemGrab {
                fin_anim: *fin_anim,
                sync_animation: false,
                grab_offset: *grab_offset,
            },
        );
        atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
        respawn_points.insert(entity, DehydrateOutOfBounds(spawner));
        transforms.insert(entity, transform);
        element_handles.insert(entity, element_handle.clone());
        hydrated.insert(entity, MapElementHydrated);
        animated_sprites.insert(entity, default());
        bodies.insert(
            entity,
            KinematicBody {
                shape: ColliderShape::Rectangle { size: *body_size },
                has_mass: true,
                can_rotate: false,
                has_friction: true,
                gravity: game_meta.physics.gravity,
                bounciness: *bounciness,
                ..default()
            },
        );
    }
}

fn update_idle_crates(
    entities: Res<Entities>,
    mut items_used: CompMut<ItemUsed>,
    mut idle_crates: CompMut<IdleCrate>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (entity, _idle_crate) in entities.iter_with(&mut idle_crates) {
        if let Some(Inv { player, .. }) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            if items_used.get(entity).is_some() {
                items_used.remove(entity);
                commands.add(PlayerCommand::set_inventory(player, None));
                commands.add(
                    move |mut idle: CompMut<IdleCrate>, mut thrown: CompMut<ThrownCrate>| {
                        idle.remove(entity);
                        thrown.insert(
                            entity,
                            ThrownCrate {
                                owner: player,
                                age: 0.0,
                                was_colliding: false,
                                crate_break_state: 0,
                            },
                        );
                    },
                );
            }
        }
    }
}

fn update_thrown_crates(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_assets: BevyAssets<ElementMeta>,
    element_handles: Comp<ElementHandle>,
    mut thrown_crates: CompMut<ThrownCrate>,
    mut commands: Commands,
    mut atlas_sprites: CompMut<AtlasSprite>,
    players: Comp<PlayerIdx>,
    collision_world: CollisionWorld,
    mut bodies: CompMut<KinematicBody>,
    mut audio_events: ResMut<AudioEvents>,
    transforms: Comp<Transform>,
    spawners: Comp<DehydrateOutOfBounds>,
    invincibles: CompMut<Invincibility>,
) {
    for (entity, (mut thrown_crate, element_handle, transform, atlas_sprite, body, spawner)) in
        entities.iter_with((
            &mut thrown_crates,
            &element_handles,
            &transforms,
            &mut atlas_sprites,
            &mut bodies,
            &spawners,
        ))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
           continue;
        };

        let BuiltinElementKind::Crate{
            breaking_anim_frames,
            breaking_atlas,
            breaking_anim_fps,
            break_timeout,
            break_sound,
            break_sound_volume,
            bounce_sound,
            bounce_sound_volume,
            crate_break_state_1,
            crate_break_state_2,
            ..
        } = &element_meta.builtin else {
            continue;
        };

        thrown_crate.age += 1.0 / crate::FPS;

        let colliding_with_tile = {
            let collider = collision_world.get_collider(entity);
            let aabb = collider.shape.compute_aabb(default());
            let width = aabb.extents().x + 2.0;
            let height = aabb.extents().y + 2.0;
            collision_world.tile_collision(
                Transform::from_translation(transform.translation),
                ColliderShape::Rectangle {
                    size: vec2(width, height),
                },
            ) != TileCollisionKind::Empty
        };

        if colliding_with_tile && !thrown_crate.was_colliding {
            thrown_crate.was_colliding = true;
            thrown_crate.crate_break_state += 1;
            audio_events.play(bounce_sound.clone(), *bounce_sound_volume);
        } else if !colliding_with_tile {
            thrown_crate.was_colliding = false;
        }

        match thrown_crate.crate_break_state {
            1 => {
                atlas_sprite.atlas = breaking_atlas.clone();
                atlas_sprite.index = *crate_break_state_1;
            }
            3 => {
                atlas_sprite.index = *crate_break_state_2;
            }
            _ => {}
        }

        let colliding_with_players = collision_world
            .actor_collisions_filtered(entity, |e| {
                    players.contains(e)
                    && invincibles.get(e).is_none()
                    && thrown_crate.was_colliding
            })
            .into_iter()
            .collect::<Vec<_>>();

        for player_entity in &colliding_with_players {

            if player_entity == &thrown_crate.owner && thrown_crate.age > 0.25 {
                commands.add(PlayerCommand::kill(
                    *player_entity,
                    Some(transform.translation.xy()),
                )); 
            }

            if thrown_crate.age > 0.25 {
                commands.add(PlayerCommand::kill(
                    *player_entity,
                    Some(transform.translation.xy()),
                )); 
            }
        }

        if !colliding_with_players.is_empty()
            || thrown_crate.age >= *break_timeout
            || thrown_crate.crate_break_state >= 4
            || body.is_on_ground && body.velocity.length_squared() < 0.1
        {
            hydrated.remove(**spawner);

            let breaking_anim_frames = *breaking_anim_frames;
            let breaking_anim_fps = *breaking_anim_fps;
            let atlas = breaking_atlas.clone();

            audio_events.play(break_sound.clone(), *break_sound_volume);

            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut animated_sprites: CompMut<AnimatedSprite>,
                      mut lifetimes: CompMut<Lifetime>,
                      mut atlas_sprites: CompMut<AtlasSprite>| {
                    let pos = transforms.get(entity).unwrap();
                    entities.kill(entity);
                    let breaking_anim_ent = entities.create();
                    atlas_sprites.insert(
                        breaking_anim_ent,
                        AtlasSprite {
                            atlas: atlas.clone(),
                            ..default()
                        },
                    );
                    animated_sprites.insert(
                        breaking_anim_ent,
                        AnimatedSprite {
                            repeat: false,
                            fps: breaking_anim_fps,
                            frames: (1..breaking_anim_frames).collect(),
                            ..default()
                        },
                    );
                    lifetimes.insert(breaking_anim_ent, Lifetime::new(1.0));
                    transforms.insert(breaking_anim_ent, *pos);
                },
            );
        }
    }
}
