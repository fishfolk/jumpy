use super::*;

pub struct DecorationPlugin;

impl Plugin for DecorationPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, hydrate_decorations);
    }
}

fn hydrate_decorations(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Handle<MapElementMeta>),
        Without<MapElementHydrated>,
    >,
    element_assets: Res<Assets<MapElementMeta>>,
) {
    // Hydrate any newly-spawned decorations
    for (entity, map_element_handle) in &non_hydrated_map_elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::AnimatedDecoration {
            atlas_handle,
            start_frame,
            end_frame,
            fps,
            ..
        } = &map_element.builtin
        {
            commands
                .entity(entity)
                .insert(MapElementHydrated)
                .insert(AnimatedSprite {
                    start: *start_frame,
                    end: *end_frame,
                    atlas: atlas_handle.inner.clone(),
                    repeat: true,
                    fps: *fps,
                    ..default()
                });
        }
    }
}
