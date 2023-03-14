use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GP9K01SC9YDTQBYK16EK7ZYD"]
pub struct Sproinger {
    pub frame: u32,
    pub sproinging: bool,
}

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut sproingers: CompMut<Sproinger>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut nav_graph: ResMut<NavGraph>,
    transforms: Comp<Transform>,
    map: Res<LoadedMap>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let mut new_sproingers = Vec::new();
    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Sproinger {
            atlas, body_size, ..
        } = &element_meta.builtin
        {
            new_sproingers.push(entity);
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: false,
                    ..default()
                },
            );
            sproingers.insert(entity, sproinger::default());
        }
    }

    // Update the navigation graph with the new sproingers
    if !new_sproingers.is_empty() {
        let mut new_graph = nav_graph.as_ref().clone();

        for ent in new_sproingers {
            let pos = transforms.get(ent).unwrap().translation;
            let node = NavNode((pos.truncate() / map.tile_size).as_ivec2());
            let sproing_to = node.above().above().above().above().above().above();

            new_graph.add_edge(
                node,
                sproing_to,
                NavGraphEdge {
                    inputs: [PlayerControl::default()].into(),
                    distance: node.distance(&sproing_to),
                },
            );
        }
        **nav_graph = Arc::new(new_graph);
    }
}

fn update(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut sproingers: CompMut<Sproinger>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    collision_world: CollisionWorld,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (entity, (sproinger, sprite)) in entities.iter_with((&mut sproingers, &mut atlas_sprites)) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Sproinger {
            sound,
            sound_volume,
            spring_velocity,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        if sproinger.sproinging {
            match sproinger.frame {
                1 => sprite.index = 2,
                4 => sprite.index = 3,
                8 => sprite.index = 4,
                12 => sprite.index = 5,
                x if x >= 20 => {
                    sprite.index = 0;
                    sproinger.sproinging = false;
                    sproinger.frame = 0;
                }
                _ => (),
            }
            sproinger.frame += 1;
        }

        for collider_ent in collision_world.actor_collisions(entity) {
            if let Some(body) = bodies.get_mut(collider_ent) {
                if body.velocity.y < *spring_velocity - body.gravity {
                    audio_events.play(sound.clone(), *sound_volume);
                    body.velocity.y = *spring_velocity;
                    sproinger.sproinging = true;
                }
            }
        }
    }
}
