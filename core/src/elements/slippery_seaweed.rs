use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GSWXSJEQXEC0TMWVBHVMHG65"]
pub struct SlipperySeaweed;

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut slippery_seaweeds: CompMut<SlipperySeaweed>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut bodies: CompMut<KinematicBody>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::SlipperySeaweed {
            atlas,
            body_size,
            start_frame,
            end_frame,
            fps,
        } = &element_meta.builtin
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            animated_sprites.insert(
                entity,
                AnimatedSprite {
                    frames: (*start_frame..*end_frame).collect(),
                    fps: *fps,
                    repeat: true,
                    ..default()
                },
            );
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: false,
                    ..default()
                },
            );
            slippery_seaweeds.insert(entity, slippery_seaweed::default());
        }
    }
}

pub fn update(
    entities: Res<Entities>,
    slippery_seaweeds: CompMut<SlipperySeaweed>,
    collision_world: CollisionWorld,
    mut player_states: CompMut<PlayerState>,
) {
    for (seaweed_ent, _) in entities.iter_with(&slippery_seaweeds) {
        for (p_ent, state) in entities.iter_with(&mut player_states) {
            if collision_world
                .actor_collisions(p_ent)
                .contains(&seaweed_ent)
            {
                state.current = key!("core::incapacitated");
                continue;
            }
        }
    }
}
