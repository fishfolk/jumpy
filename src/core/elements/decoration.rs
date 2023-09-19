use crate::prelude::*;

pub fn install(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, hydrate);
}

fn hydrate(
    entities: Res<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    for entity in entities.iter_with_bitset(&not_hydrated_bitset) {
        let element_handle = element_handles.get(entity).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(AnimatedDecorationMeta {
            start_frame,
            end_frame,
            fps,
            atlas,
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(entity, MapElementHydrated);
            atlas_sprites.insert(
                entity,
                AtlasSprite {
                    atlas: *atlas,
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
