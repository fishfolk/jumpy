use bevy_ecs_tilemap::TilemapPlugin;

use crate::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin);
    }
}
