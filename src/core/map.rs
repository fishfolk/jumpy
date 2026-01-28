//! Map and navigation mesh implementation.

use std::{
    cmp::{max, min},
    collections::VecDeque,
};

use super::physics::collisions::{CollisionWorld, TileCollisionKind, TileDynamicCollider};
use crate::prelude::*;

pub fn install(session: &mut SessionBuilder) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, spawn_map)
        .add_system_to_stage(CoreStage::First, handle_out_of_bounds_players);
}

/// Resource containing the map metadata for this game session.
#[derive(Clone, HasSchema, Deref, DerefMut, Default)]
pub struct LoadedMap(pub Arc<MapMeta>);

/// Resource indicating whether the map has been spawned.
#[derive(Clone, HasSchema, Default, Deref, DerefMut)]
pub struct MapSpawned(pub bool);

/// The Z depth of the deepest map layer.
pub const MAP_LAYERS_MIN_DEPTH: f32 = -900.0;
/// The Z depth in between each map layer.
pub const MAP_LAYERS_GAP_DEPTH: f32 = 10.0;

/// Helper for getting the z-depth of the map layer with the given index.
pub fn z_depth_for_map_layer(layer_idx: u32) -> f32 {
    // We start map layers at -900 and for ever layer we place a gap of 10 units in between
    MAP_LAYERS_MIN_DEPTH + layer_idx as f32 * MAP_LAYERS_GAP_DEPTH
}

/// Resource containing essential the map metadata for the map once spawned. This allows the
/// complete map metadata to be re-constructed from the world after the map has been spawned and
/// potentially modified.
#[derive(HasSchema, Clone)]
pub struct SpawnedMapMeta {
    pub name: Ustr,
    pub background: Arc<BackgroundMeta>,
    pub background_color: Color,
    pub grid_size: UVec2,
    pub tile_size: Vec2,
    pub layer_names: Arc<[Ustr]>,
}

impl Default for SpawnedMapMeta {
    fn default() -> Self {
        Self {
            name: "".into(),
            background: default(),
            background_color: default(),
            grid_size: default(),
            tile_size: default(),
            layer_names: Arc::new([]),
        }
    }
}

/// Component containing the map layer that an entity is associated to.
///
/// This is used when exporting the world to `MapMeta` to decide which layer to put an element or
/// tile layer in.
#[derive(HasSchema, Clone, Copy, Default)]
pub struct SpawnedMapLayerMeta {
    /// The layer index of the layer that the element belongs to in the map.
    pub layer_idx: u32,
}

/// The map navigation graph resource.
#[derive(Clone, Debug, Deref, DerefMut, HasSchema, Default)]
pub struct NavGraph(pub Arc<NavGraphInner>);

/// The inner graph type of [`NavGraph`].
pub type NavGraphInner = petgraph::graphmap::DiGraphMap<NavNode, NavGraphEdge>;

/// The type of nodes in the map navigation graph.
///
/// This is merely a wrapper around [`UVec2`] to add an [`Ord`] implementation.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Deref, DerefMut)]
pub struct NavNode(pub IVec2);

impl NavNode {
    /// Calculates the Pythagorean distance between two nodes.
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = (max(self.x, other.x) - min(self.x, other.x)) as f32;
        let dy = (max(self.y, other.y) - min(self.y, other.y)) as f32;
        (dx * dx) + (dy * dy)
    }

    pub fn right(&self) -> NavNode {
        NavNode(self.0 + ivec2(1, 0))
    }

    pub fn above(&self) -> NavNode {
        NavNode(self.0 + ivec2(0, 1))
    }

    pub fn left(&self) -> NavNode {
        NavNode(self.0 - ivec2(1, 0))
    }

    pub fn below(&self) -> NavNode {
        NavNode(self.0 - ivec2(0, 1))
    }
}

impl From<IVec2> for NavNode {
    fn from(v: IVec2) -> Self {
        Self(v)
    }
}
impl From<NavNode> for IVec2 {
    fn from(v: NavNode) -> Self {
        v.0
    }
}
impl std::cmp::Ord for NavNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let xcmp = self.0.x.cmp(&other.0.x);
        if xcmp == std::cmp::Ordering::Equal {
            self.0.y.cmp(&other.0.y)
        } else {
            xcmp
        }
    }
}
impl std::cmp::PartialOrd for NavNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents the way to get from one tile to another tile in the navigation graph.
#[derive(Debug, Clone)]
pub struct NavGraphEdge {
    /// The sequence of inputs for each frame, required to get to the connected tile.
    pub inputs: VecDeque<PlayerControl>,
    /// The distance to the connected tile. This is used as the heuristic for pathfinding.
    pub distance: f32,
}

fn spawn_map(
    mut commands: Commands,
    mut entities: ResMutInit<Entities>,
    mut clear_color: ResMutInit<ClearColor>,
    assets: Res<AssetServer>,
    map: Res<LoadedMap>,
    mut map_spawned: ResMutInit<MapSpawned>,
    mut tiles: CompMut<Tile>,
    mut tile_layers: CompMut<TileLayer>,
    mut transforms: CompMut<Transform>,
    mut element_handles: CompMut<ElementHandle>,
    mut tile_collisions: CompMut<TileCollisionKind>,
    mut tile_dynamic_colliders: CompMut<TileDynamicCollider>,
    mut parallax_bg_sprites: CompMut<ParallaxBackgroundSprite>,
    mut sprites: CompMut<Sprite>,
    mut nav_graph: ResMutInit<NavGraph>,
    mut cameras: CompMut<Camera>,
    mut camera_shakes: CompMut<CameraShake>,
    mut camera_states: CompMut<CameraState>,
    mut spawned_map_layer_metas: CompMut<SpawnedMapLayerMeta>,
    mut spawned_map_meta: ResMutInit<SpawnedMapMeta>,
) {
    if map_spawned.0 {
        return;
    }

    // Fill in the spawned map metadata
    *spawned_map_meta = SpawnedMapMeta {
        name: map.name,
        background: Arc::new(map.background.clone()),
        background_color: map.background_color,
        grid_size: map.grid_size,
        tile_size: map.tile_size,
        layer_names: map.layers.iter().map(|x| x.id).collect(),
    };

    // Spawn the camera
    {
        let ent = entities.create();
        camera_shakes.insert(
            ent,
            CameraShake {
                center: (map.tile_size * (map.grid_size / 2).as_vec2()).extend(0.0),
                ..CameraShake::new(2.0, vec2(20.0, 20.0), 1.0, 1.0)
            },
        );
        cameras.insert(ent, default());
        transforms.insert(ent, default());
        camera_states.insert(ent, default());
    }

    map_spawned.0 = true;
    **clear_color = map.background_color;

    // Load the navigation graph
    nav_graph.0 = create_nav_graph(&map);

    // Spawn parallax backgrounds
    for layer in &map.background.layers {
        for i in -1..=1 {
            let ent = entities.create();
            sprites.insert(
                ent,
                Sprite {
                    image: layer.image,
                    ..default()
                },
            );
            transforms.insert(ent, default());
            parallax_bg_sprites.insert(
                ent,
                ParallaxBackgroundSprite {
                    idx: i,
                    meta: layer.clone(),
                },
            );
        }
    }

    // Load tiles
    for (layer_idx, layer) in map.layers.iter().enumerate() {
        let layer_idx = layer_idx as u32;
        let layer_z = z_depth_for_map_layer(layer_idx);

        if let Set(tilemap) = layer.tilemap {
            let mut tile_layer = TileLayer::new(map.grid_size, map.tile_size, tilemap);

            for tile_meta in &layer.tiles {
                let tile_ent = entities.create();
                tile_layer.set(tile_meta.pos, Some(tile_ent));
                tiles.insert(
                    tile_ent,
                    Tile {
                        idx: tile_meta.idx,
                        ..default()
                    },
                );
                // TODO: Due to a bug in the way that collisions are handled in the
                // `physics/collisions.rs` file, having an empty collision isn't equivalent to not
                // having a collision component.
                //
                // We should fix this so that we can insert an empty tile collision kind and have that
                // work properly. For now, though, just not adding the tile collider component behaves
                // properly.
                if tile_meta.collision != TileCollisionKind::Empty {
                    tile_collisions.insert(tile_ent, tile_meta.collision);
                }

                let atlas = assets.get(tilemap);
                let optional_tile_colliders = &atlas.tile_collision;
                // If metadata has valid dynamics collider, create component for it.
                if let Some(extra_collider) =
                    optional_tile_colliders.get(&format!("{}", tile_meta.idx))
                {
                    if extra_collider.has_area() {
                        // clamp collider between 0.0, 1.0
                        let extra_collider = extra_collider.clamped_values();

                        // scale min/max between 0 and tile size.
                        let tile_size = spawned_map_meta.tile_size;
                        let min = extra_collider.min * tile_size;
                        let max = extra_collider.max * tile_size;

                        // compute offset from center of tile
                        let collider_center = min + (max - min) * 0.5;
                        let offset =
                            collider_center - Vec2::new(tile_size.x * 0.5, tile_size.y * 0.5);

                        let extents = max - min;
                        let shape = ColliderShape::Rectangle { size: extents };

                        tile_dynamic_colliders
                            .insert(tile_ent, TileDynamicCollider { shape, offset });
                    }
                }
            }
            let layer_ent = entities.create();
            spawned_map_layer_metas.insert(layer_ent, SpawnedMapLayerMeta { layer_idx });
            tile_layers.insert(layer_ent, tile_layer);
            transforms.insert(
                layer_ent,
                Transform::from_translation(Vec3::new(0.0, 0.0, layer_z)),
            );
        }

        // Spawn the elements
        for element_meta in &layer.elements {
            let element_ent = entities.create();

            spawned_map_layer_metas.insert(element_ent, SpawnedMapLayerMeta { layer_idx });
            transforms.insert(
                element_ent,
                Transform::from_translation(element_meta.pos.extend(layer_z)),
            );
            element_handles.insert(element_ent, ElementHandle(element_meta.element));
        }
    }

    // Update collision world with map tiles
    commands.add(|mut collision_world: CollisionWorld| {
        collision_world.update_tiles();
    });
}

fn handle_out_of_bounds_players(
    entities: Res<Entities>,
    mut commands: Commands,
    transforms: CompMut<Transform>,
    player_indexes: Comp<PlayerIdx>,
    map: Res<LoadedMap>,
) {
    for (player_ent, (_player_idx, transform)) in entities.iter_with((&player_indexes, &transforms))
    {
        if map.is_out_of_bounds(&transform.translation) {
            commands.add(PlayerCommand::kill(player_ent, None));
        }
    }
}

/// Helper method to create a navigation graph from the map metadata.
fn create_nav_graph(meta: &MapMeta) -> Arc<NavGraphInner> {
    // Load the navigation graph
    let mut graph = NavGraphInner::default();

    // Initialize set of traversable tiles, assuming all tiles are traversable
    for x in 0..meta.grid_size.x as i32 {
        for y in 0..meta.grid_size.y as i32 {
            graph.add_node(NavNode(ivec2(x, y)));
        }
    }

    // Find all solid tiles and remove them from the traversable tiles list, while also recording
    // the jump-through tiles.
    let mut semi_solids = HashSet::default();
    for layer in &meta.layers {
        for tile in &layer.tiles {
            if tile.collision == TileCollisionKind::JumpThrough {
                semi_solids.insert(NavNode(tile.pos.as_ivec2()));
            } else if tile.collision != TileCollisionKind::Empty {
                graph.remove_node(NavNode(tile.pos.as_ivec2()));
            }
        }
    }

    // Calculate possible movements from every node
    macro_rules! is_solid {
        ($node:expr) => {
            !graph.contains_node($node) || semi_solids.contains(&$node)
        };
    }

    for node in graph.nodes().collect::<Vec<_>>() {
        // walk left or right along the ground
        let has_ground = is_solid!(node.below());
        let maybe_has_ground =
            has_ground || is_solid!(node.below().left()) || is_solid!(node.below().right());

        /////////////////
        // Grounded
        /////////////////

        if maybe_has_ground {
            // Moving Right
            let right = node.right();
            if graph.contains_node(right) {
                graph.add_edge(
                    node,
                    right,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            moving: true,
                            move_direction: vec2(1.0, 0.0),
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&right),
                    },
                );
            }

            // Moving Left
            let left = node.left();
            if graph.contains_node(left) {
                graph.add_edge(
                    node,
                    left,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            moving: true,
                            move_direction: vec2(-1.0, 0.0),
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&left),
                    },
                );
            }
        }

        if has_ground {
            /////////////////
            // JUMPING
            /////////////////
            let above1 = node.above();
            let above2 = above1.above();
            let above3 = above2.above();
            let contains_above1 = graph.contains_node(above1);
            let contains_above2 = graph.contains_node(above2);
            let contains_above3 = graph.contains_node(above3);

            if contains_above1 {
                // Jump staight up
                graph.add_edge(
                    node,
                    above1,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            jump_just_pressed: true,
                            jump_pressed: true,
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&above1),
                    },
                );
            }
            if contains_above2 && contains_above1 {
                // Jump staight up
                graph.add_edge(
                    node,
                    above2,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            jump_just_pressed: true,
                            jump_pressed: true,
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&above2),
                    },
                );
            }
            if contains_above3 && contains_above2 && contains_above1 {
                // Jump staight up
                graph.add_edge(
                    node,
                    above2,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            jump_just_pressed: true,
                            jump_pressed: true,
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&above3),
                    },
                );
            }

            // Jump up and left
            let above3l2 = above3.left().left();
            let above2l = above2.left();
            let contains_above2l = graph.contains_node(above2l);
            let contains_above3l2 = graph.contains_node(above3l2);
            if contains_above3l2 && contains_above2 && contains_above3 && contains_above2l {
                graph.add_edge(
                    node,
                    above3l2,
                    NavGraphEdge {
                        inputs: std::iter::repeat_n(
                            PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            20,
                        )
                        .collect(),
                        distance: node.distance(&above3l2),
                    },
                );
            }
            let above3l3 = above3.left().left().left();
            if graph.contains_node(above3l3)
                && graph.contains_node(above3.left())
                && contains_above3l2
                && contains_above2
                && contains_above3
                && contains_above2l
            {
                graph.add_edge(
                    node,
                    above3l3,
                    NavGraphEdge {
                        inputs: std::iter::repeat_n(
                            PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            20,
                        )
                        .collect(),
                        distance: node.distance(&above3l3),
                    },
                );
            }

            // Jump up and right
            let above3r2 = above3.right().right();
            let above2r = above2.right();
            let contains_above2r = graph.contains_node(above2r);
            let contains_above3r2 = graph.contains_node(above3r2);
            if contains_above3r2 && contains_above2 && contains_above3 && contains_above2r {
                graph.add_edge(
                    node,
                    above3r2,
                    NavGraphEdge {
                        inputs: std::iter::repeat_n(
                            PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            20,
                        )
                        .collect(),
                        distance: node.distance(&above3r2),
                    },
                );
            }
            let above3r3 = above3.right().right().right();
            if graph.contains_node(above3r3)
                && graph.contains_node(above3.right())
                && contains_above3r2
                && contains_above2
                && contains_above3
                && contains_above2r
            {
                graph.add_edge(
                    node,
                    above3r3,
                    NavGraphEdge {
                        inputs: std::iter::repeat_n(
                            PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            20,
                        )
                        .collect(),
                        distance: node.distance(&above3r3),
                    },
                );
            }
        }

        /////////////////
        // Falling Down
        /////////////////

        // Fall straight down
        let below = node.below();
        if graph.contains_node(below) {
            if semi_solids.contains(&below) {
                graph.add_edge(
                    node,
                    below,
                    NavGraphEdge {
                        inputs: [
                            PlayerControl {
                                move_direction: vec2(0.0, -1.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            default(),
                            default(),
                            default(),
                            default(),
                        ]
                        .into(),
                        distance: node.distance(&below),
                    },
                );
            } else {
                graph.add_edge(
                    node,
                    below,
                    NavGraphEdge {
                        inputs: [PlayerControl::default()].into(),
                        distance: node.distance(&below),
                    },
                );
            }
        }

        // Fall diagonally down right
        let below_right = node.below().right();
        if graph.contains_node(below_right) {
            if semi_solids.contains(&below_right) {
                graph.add_edge(
                    node,
                    below_right,
                    NavGraphEdge {
                        inputs: [
                            PlayerControl {
                                move_direction: vec2(1.0, -1.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(1.0, -1.0),
                                jump_pressed: true,
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                ..default()
                            },
                        ]
                        .into(),
                        distance: node.distance(&below_right),
                    },
                );
            } else {
                graph.add_edge(
                    node,
                    below_right,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            move_direction: vec2(1.0, 0.0),
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&below_right),
                    },
                );
            }
        }
        // Fall diagonally down left
        let below_left = node.below().left();
        if graph.contains_node(below_left) {
            if semi_solids.contains(&below_left) {
                graph.add_edge(
                    node,
                    below_left,
                    NavGraphEdge {
                        inputs: [
                            PlayerControl {
                                move_direction: vec2(-1.0, -1.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(-1.0, -1.0),
                                jump_pressed: true,
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                ..default()
                            },
                            PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                ..default()
                            },
                        ]
                        .into(),
                        distance: node.distance(&below_left),
                    },
                );
            } else {
                graph.add_edge(
                    node,
                    below_left,
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            move_direction: vec2(-1.0, 0.0),
                            ..default()
                        }]
                        .into(),
                        distance: node.distance(&below_left),
                    },
                );
            }
        }

        // Slow fall right
        let far_right_below = node.right().right().right().right().below();
        let path = [
            node.right(),
            node.right().right(),
            node.right().right().right(),
            node.right().right().right().right(),
            far_right_below,
        ];
        if path.iter().all(|x| graph.contains_node(*x)) {
            graph.add_edge(
                node,
                far_right_below,
                NavGraphEdge {
                    inputs: std::iter::repeat_n(
                        PlayerControl {
                            move_direction: vec2(1.0, 0.0),
                            jump_pressed: true,
                            ..default()
                        },
                        20,
                    )
                    .collect(),
                    // Bias against using this move because it doesn't always work, by adding an
                    // extra distance.
                    distance: node.distance(&far_right_below) + 1.0,
                },
            );
        }
        // Slow fall left
        let far_left_below = node.left().left().left().left().below();
        let path = [
            node.left(),
            node.left().left(),
            node.left().left().left(),
            node.left().left().left().left(),
            far_left_below,
        ];
        if path.iter().all(|x| graph.contains_node(*x)) {
            graph.add_edge(
                node,
                far_left_below,
                NavGraphEdge {
                    inputs: std::iter::repeat_n(
                        PlayerControl {
                            move_direction: vec2(-1.0, 0.0),
                            jump_pressed: true,
                            ..default()
                        },
                        20,
                    )
                    .collect(),
                    // Bias against using this move because it doesn't always work, by adding an
                    // extra distance.
                    distance: node.distance(&far_left_below) + 1.0,
                },
            );
        }
    }

    Arc::new(graph)
}
