use bevy::{ecs::system::SystemParam, render::view::RenderLayers, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;
use bevy_mod_js_scripting::{ActiveScripts, JsScript};
use bevy_parallax::ParallaxResource;
use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{
    camera::GameRenderLayers,
    metadata::{MapElementMeta, MapLayerKind, MapLayerMeta, MapMeta},
    prelude::*,
};

pub mod grid;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapScripts>().add_plugin(TilemapPlugin);
    }
}

/// Contains the scripts that have been added for the currently loaded map
#[derive(Deref, DerefMut, Default)]
pub struct MapScripts(pub HashSet<Handle<JsScript>>);

#[derive(SystemParam)]
pub struct SpawnMapParams<'w, 's> {
    commands: Commands<'w, 's>,
    map_assets: ResMut<'w, Assets<MapMeta>>,
    parallax: ResMut<'w, ParallaxResource>,
    windows: Res<'w, Windows>,
    asset_server: Res<'w, AssetServer>,
    texture_atlas_assets: ResMut<'w, Assets<TextureAtlas>>,
    element_assets: ResMut<'w, Assets<MapElementMeta>>,
    active_scripts: ResMut<'w, ActiveScripts>,
    map_scripts: ResMut<'w, MapScripts>,
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
    let mut map_children = Vec::new();

    // Spawn the grid
    let grid_entity = params
        .commands
        .spawn()
        .insert(Name::new("Map Grid"))
        .insert(MapGridView)
        .insert_bundle(grid)
        .insert(RenderLayers::layer(GameRenderLayers::EDITOR))
        .id();
    map_children.push(grid_entity);

    // Clear any previously loaded map scripts
    for script in params.map_scripts.drain() {
        params.active_scripts.remove(&script);
    }

    // Spawn map layers
    for (i, layer) in map.layers.iter().enumerate() {
        let layer: &MapLayerMeta = layer;
        let layer_id = &layer.id;

        match &layer.kind {
            MapLayerKind::Tile(tile_layer) => {
                let layer_entity = params
                    .commands
                    .spawn()
                    .insert(Name::new(format!("Map Layer: {layer_id}")))
                    .id();
                let mut storage = TileStorage::empty(tilemap_size);

                let mut tile_entities = Vec::new();
                for tile in &tile_layer.tiles {
                    let tile_pos = TilePos {
                        x: tile.pos.x,
                        y: tile.pos.y,
                    };

                    let half_tile_x = map.tile_size.x as f32 / 2.0;
                    let half_tile_y = map.tile_size.y as f32 / 2.0;
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
                        .insert(RigidBody::Fixed)
                        .insert(Collider::cuboid(half_tile_x, half_tile_y))
                        .insert_bundle(TransformBundle {
                            local: Transform::from_xyz(
                                half_tile_x + map.tile_size.x as f32 * tile_pos.x as f32,
                                half_tile_y + map.tile_size.y as f32 * tile_pos.y as f32,
                                0.0,
                            ),
                            ..default()
                        })
                        .id();

                    storage.set(&tile_pos, Some(tile_entity));

                    tile_entities.push(tile_entity);
                }

                let tile_layer = params
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
                    })
                    .push_children(&tile_entities)
                    .id();

                map_children.push(tile_layer);
            }
            MapLayerKind::Element(element_layer) => {
                for element in &element_layer.elements {
                    let element_meta = params
                        .element_assets
                        .get(&element.element_handle)
                        .unwrap()
                        .clone();
                    params
                        .active_scripts
                        .insert(element_meta.script_handle.clone_weak());
                    params
                        .map_scripts
                        .insert(element_meta.script_handle.clone_weak());

                    let element_name = &element_meta.name;

                    let entity = params
                        .commands
                        .spawn()
                        .insert(Name::new(format!(
                            "Map Element ( {layer_id} ): {element_name}"
                        )))
                        .insert_bundle(VisibilityBundle::default())
                        .insert_bundle(TransformBundle {
                            local: Transform::from_xyz(
                                element.pos.x,
                                element.pos.y,
                                -100.0 + i as f32,
                            ),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn()
                                .insert(Name::new("Map Element Debug Rect"))
                                .insert(RenderLayers::layer(GameRenderLayers::EDITOR))
                                .insert_bundle(GeometryBuilder::build_as(
                                    &Rectangle {
                                        extents: element_meta.editor_size,
                                        ..default()
                                    },
                                    #[allow(const_item_mutation)]
                                    DrawMode::Stroke(StrokeMode::new(
                                        *Color::GREEN.set_a(0.5),
                                        0.5,
                                    )),
                                    default(),
                                ));
                        })
                        .insert(element_meta)
                        .id();

                    map_children.push(entity)
                }
            }
        }
    }

    params
        .commands
        .entity(map_entity)
        .push_children(&map_children);
}
