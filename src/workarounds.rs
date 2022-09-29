//! This module contains temporary workarounds that should be removed once proper solutions can be
//! put in place.

use bevy::render::camera::CameraUpdateSystem;

use crate::prelude::*;

pub struct WorkaroundsPlugin;

impl Plugin for WorkaroundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_camera_projection_when_camera_changes.before(CameraUpdateSystem),
        );
    }
}

/// Workaround for bevy not updating camera projection when viewport changes:
/// <https://github.com/bevyengine/bevy/issues/5944>
fn update_camera_projection_when_camera_changes(
    mut changed_cameras: Query<&mut OrthographicProjection, Changed<Camera>>,
) {
    // For every changed camera
    for mut projection in &mut changed_cameras {
        // Force a change to the projection in order to trigger a re-building of the camera
        // projection for the new viewport.
        projection.set_changed();
    }
}
