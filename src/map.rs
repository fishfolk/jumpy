use bevy::{ecs::system::SystemParam, render::view::RenderLayers};
use bevy_ecs_tilemap::prelude::*;
use bevy_parallax::ParallaxResource;
use bevy_prototype_lyon::prelude::*;

use crate::{
    camera::GameRenderLayers,
    metadata::{MapLayerKind, MapLayerMeta, MapMeta},
    prelude::*,
};

pub mod grid;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin);
    }
}

#[derive(SystemParam)]
pub struct SpawnMapParams<'w, 's> {
    commands: Commands<'w, 's>,
    map_assets: ResMut<'w, Assets<MapMeta>>,
    parallax: ResMut<'w, ParallaxResource>,
    windows: Res<'w, Windows>,
    asset_server: Res<'w, AssetServer>,
    texture_atlas_assets: ResMut<'w, Assets<TextureAtlas>>,
}

/// Marker component for the map grid
#[derive(Component)]
pub struct MapGridView;

pub enum MapSpawnSource {
    Handle(Handle<MapMeta>),
    Meta(MapMeta),
}

pub fn spawn_map(params: &mut SpawnMapParams, source: &MapSpawnSource) {
    let map = match &source {
        MapSpawnSource::Handle(handle) => params.map_assets.get(handle).unwrap(),
        MapSpawnSource::Meta(meta) => meta,
    };

    let grid = GeometryBuilder::build_as(
        &grid::Grid {
            grid_size: map.grid_size,
            tile_size: map.tile_size,
        },
        DrawMode::Stroke(StrokeMode::new(Color::rgba(0.8, 0.8, 0.8, 0.25), 0.5)),
        default(),
    );

    let window = params.windows.primary();
    *params.parallax = map.get_parallax_resource();
    params.parallax.window_size = Vec2::new(window.width(), window.height());
    params.parallax.create_layers(
        &mut params.commands,
        &params.asset_server,
        &mut params.texture_atlas_assets,
    );

    params
        .commands
        .insert_resource(ClearColor(map.background_color.into()));

    let tilemap_size = TilemapSize {
        x: map.grid_size.x,
        y: map.grid_size.y,
    };

    let map_entity = params
        .commands
        .spawn()
        .insert(Name::new("Map Entity"))
        .insert(map.clone())
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::default())
        .id();
    let mut children = Vec::new();

    // Spawn the grid
    let grid_entity = params
        .commands
        .spawn()
        .insert(Name::new("Map Grid"))
        .insert(MapGridView)
        .insert_bundle(grid)
        .insert(RenderLayers::layer(GameRenderLayers::EDITOR))
        .id();
    children.push(grid_entity);

    // Spawn map layers
    for (i, layer) in map.layers.iter().enumerate() {
        let layer: &MapLayerMeta = layer;

        match &layer.kind {
            MapLayerKind::Tile(tile_layer) => {
                let layer_entity = params
                    .commands
                    .spawn()
                    .insert(Name::new(format!("Map Layer: {}", layer.id)))
                    .id();
                let mut storage = TileStorage::empty(tilemap_size);

                for tile in &tile_layer.tiles {
                    let tile_pos = TilePos {
                        x: tile.pos.x,
                        y: tile.pos.y,
                    };

                    let tile_entity = params
                        .commands
                        .spawn()
                        .insert(Name::new(format!(
                            "Map Tile: {}: ( {} x {} )",
                            layer.id, tile.pos.x, tile.pos.y,
                        )))
                        .insert_bundle(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(layer_entity),
                            texture: TileTexture(tile.idx),
                            ..default()
                        })
                        .id();

                    storage.set(&tile_pos, Some(tile_entity));
                }

                params
                    .commands
                    .entity(layer_entity)
                    .insert_bundle(TilemapBundle {
                        grid_size: TilemapGridSize {
                            x: map.grid_size.x as f32,
                            y: map.grid_size.y as f32,
                        },
                        tile_size: TilemapTileSize {
                            x: map.tile_size.x as f32,
                            y: map.tile_size.y as f32,
                        },
                        texture: TilemapTexture(tile_layer.tilemap_handle.clone()),
                        storage,
                        transform: Transform::from_xyz(0.0, 0.0, -100.0 + i as f32),
                        ..default()
                    });
            }
            MapLayerKind::Element(_) => (),
        }
    }

    params.commands.entity(map_entity).push_children(&children);
}
