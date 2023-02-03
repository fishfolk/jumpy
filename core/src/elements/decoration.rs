use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate);
}

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::AnimatedDecoration {
            start_frame,
            end_frame,
            fps,
            atlas,
        } = &element_meta.builtin
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(
                entity,
                AtlasSprite {
                    atlas: atlas.clone(),
                    ..default()
                },
            );
            animated_sprites.insert(
                entity,
                AnimatedSprite {
                    frames: (*start_frame..*end_frame).collect(),
                    fps: *fps,
                    repeat: true,
                    ..default()
                },
            );
        }
    }
}
