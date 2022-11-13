use bevy::{render::view::RenderLayers, utils::HashSet};
use bevy_ecs_tilemap::prelude::*;
use bevy_mod_js_scripting::{ActiveScripts, JsScript};
use bevy_parallax::ParallaxResource;
use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};

use crate::{
    camera::GameRenderLayers,
    metadata::{MapElementMeta, MapLayerKind, MapLayerMeta, MapMeta},
    name::EntityName,
    physics::collisions::{CollisionLayerTag, TileCollision},
    prelude::*,
};

pub mod grid;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .init_resource::<MapScripts>()
            .add_system(hydrate_maps)
            .extend_rollback_plugin(|plugin| {
                plugin.register_rollback_type::<MapElementScriptAcknowledged>()
            });
    }
}

/// Marker component indicating that a map element load event has been handled by the map element's
/// script.
#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct MapElementScriptAcknowledged;

/// Contains the scripts that have been added for the currently loaded map
#[derive(Deref, DerefMut, Default)]
pub struct MapScripts(pub HashSet<Handle<JsScript>>);

/// Marker component for the map grid
#[derive(Component)]
pub struct MapGridView;

pub fn hydrate_maps(
    mut commands: Commands,
    mut parallax: ResMut<ParallaxResource>,
    map_assets: Res<Assets<MapMeta>>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
    element_assets: ResMut<Assets<MapElementMeta>>,
    mut active_scripts: ResMut<ActiveScripts>,
    mut map_scripts: ResMut<MapScripts>,
    mut rids: ResMut<RollbackIdProvider>,
    unspawned_maps: Query<(Entity, &AssetHandle<MapMeta>), Without<MapMeta>>,
) {
    for (map_entity, map_handle) in &unspawned_maps {
        let map = map_assets.get(map_handle).expect("Map asset not loaded");
        let grid = GeometryBuilder::build_as(
            &grid::Grid {
                grid_size: map.grid_size,
                tile_size: map.tile_size,
            },
            DrawMode::Stroke(StrokeMode::new(Color::rgba(0.8, 0.8, 0.8, 0.25), 0.5)),
            default(),
        );

        let window = windows.primary();
        *parallax = map.get_parallax_resource();
        parallax.window_size = Vec2::new(window.width(), window.height());
        parallax.create_layers(&mut commands, &asset_server, &mut texture_atlas_assets);

        commands.insert_resource(ClearColor(map.background_color.into()));

        let tilemap_size = TilemapSize {
            x: map.grid_size.x,
            y: map.grid_size.y,
        };

        commands
            .entity(map_entity)
            .insert(map.clone())
            .insert(Name::new("Map"))
            .insert(map.clone())
            .insert_bundle(VisibilityBundle::default())
            .insert_bundle(TransformBundle::default());
        let mut map_children = Vec::new();

        // Spawn the grid
        let grid_entity = commands
            .spawn()
            .insert(Name::new("Grid"))
            .insert(MapGridView)
            .insert_bundle(grid)
            .insert(RenderLayers::layer(GameRenderLayers::EDITOR))
            .id();
        map_children.push(grid_entity);

        // Clear any previously loaded map scripts
        for script in map_scripts.drain() {
            active_scripts.remove(&script);
        }

        // Spawn map layers
        for (i, layer) in map.layers.iter().enumerate() {
            let layer: &MapLayerMeta = layer;
            let layer_id = &layer.id;

            match &layer.kind {
                MapLayerKind::Tile(tile_layer) => {
                    let layer_entity = commands
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
                        let mut tile_entity_commands = commands.spawn();

                        tile_entity_commands
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
                            .insert_bundle(TransformBundle {
                                local: Transform::from_xyz(
                                    half_tile_x + map.tile_size.x as f32 * tile_pos.x as f32,
                                    half_tile_y + map.tile_size.y as f32 * tile_pos.y as f32,
                                    0.0,
                                ),
                                ..default()
                            });

                        if tile.jump_through {
                            tile_entity_commands.insert(TileCollision::JumpThrough);
                        } else {
                            tile_entity_commands.insert(TileCollision::Solid);
                        }
                        let tile_entity = tile_entity_commands.id();

                        storage.set(&tile_pos, Some(tile_entity));

                        tile_entities.push(tile_entity);
                    }

                    let mut layer_commands = commands.entity(layer_entity);

                    layer_commands
                        .insert_bundle(TilemapBundle {
                            grid_size: TilemapGridSize {
                                x: map.grid_size.x as f32,
                                y: map.grid_size.y as f32,
                            },
                            tile_size: TilemapTileSize {
                                x: map.tile_size.x as f32,
                                y: map.tile_size.y as f32,
                            },
                            texture: TilemapTexture(tile_layer.tilemap_handle.inner.clone_weak()),
                            storage,
                            transform: Transform::from_xyz(0.0, 0.0, -100.0 + i as f32),
                            ..default()
                        })
                        .push_children(&tile_entities);

                    if tile_layer.has_collision {
                        layer_commands.insert(CollisionLayerTag::default());
                    }

                    let tile_layer = layer_commands.id();

                    map_children.push(tile_layer);
                }
                MapLayerKind::Element(element_layer) => {
                    for element in &element_layer.elements {
                        let element_meta =
                            element_assets.get(&element.element_handle).unwrap().clone();
                        for script_handle in &element_meta.script_handles {
                            active_scripts.insert(script_handle.inner.clone_weak());
                            map_scripts.insert(script_handle.inner.clone_weak());
                        }

                        let element_name = &element_meta.name;

                        let entity = commands
                            .spawn()
                            .insert(EntityName(format!(
                                "Map Element ( {layer_id} ): {element_name}"
                            )))
                            .insert(Visibility::default())
                            .insert(ComputedVisibility::default())
                            .insert(Transform::from_xyz(
                                element.pos.x,
                                element.pos.y,
                                -100.0 + i as f32,
                            ))
                            .insert(GlobalTransform::default())
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
                            .insert(Rollback::new(rids.next_id()))
                            .id();
                        map_children.push(entity)
                    }
                }
            }
        }

        commands.entity(map_entity).push_children(&map_children);
    }
}
