use crate::prelude::*;

pub fn install(session: &mut CoreSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01H3DBQHJGFKT8G81BMEMN9FN6"]
pub struct Spike;

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut spikes: CompMut<Spike>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let mut new_spikes = Vec::new();
    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Spike {
            atlas,
            body_size,
            start_frame,
            end_frame,
            fps,
            ..
        } = &element_meta.builtin
        {
            new_spikes.push(entity);
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
            spikes.insert(entity, spike::default());
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut spikes: CompMut<Spike>,
    collision_world: CollisionWorld,
    player_indexes: Comp<PlayerIdx>,
    invincibles: CompMut<Invincibility>,
    mut commands: Commands,
    transforms: Comp<Transform>,
) {
    for (entity, (_, pos)) in entities.iter_with((&mut spikes, &transforms)) {
        collision_world
            .actor_collisions_filtered(entity, |e| {
                player_indexes.contains(e) && invincibles.get(e).is_none()
            })
            .into_iter()
            .for_each(|player| {
                commands.add(PlayerCommand::kill(player, Some(pos.translation.xy())));
            });
    }
}
