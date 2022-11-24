use super::*;

pub struct DecorationPlugin;

impl Plugin for DecorationPlugin {
    fn build(&self, app: &mut App) {
        app.extend_rollback_schedule(|schedule| {
            schedule.add_system_to_stage(RollbackStage::PreUpdate, hydrate_decorations);
        });
    }
}

fn hydrate_decorations(
    mut commands: Commands,
    non_hydrated_map_elements: Query<(Entity, &MapElementMeta), Without<MapElementHydrated>>,
) {
    // Hydrate any newly-spawned decorations
    for (entity, map_element) in &non_hydrated_map_elements {
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
