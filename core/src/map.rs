use std::collections::VecDeque;

use ::bevy::utils::HashSet;

use crate::prelude::{collisions::TileCollision, *};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, spawn_map)
        .add_system_to_stage(CoreStage::First, handle_out_of_bounds_players_and_items);
}

/// Resource containing the map metadata for this game session.
#[derive(Clone, TypeUlid, Deref, DerefMut, Default)]
#[ulid = "01GP2H6K9H3JEEMXFCKV4TGMWZ"]
pub struct MapHandle(pub Handle<MapMeta>);

/// Resource indicating whether the map has been spawned.
#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP3Z38HKE37JB6GRHHPPTY38"]
pub struct MapSpawned(pub bool);

#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP9NY0Y50Y2A8M4A7E9NN8VE"]
pub struct MapRespawnPoint(pub Vec3);

/// The map navigation graph resource.
#[derive(Clone, Debug, Deref, DerefMut, TypeUlid, Default)]
#[ulid = "01GQWP4QG11NBVX3M289TXAK6W"]
pub struct NavGraph(pub Option<Arc<NavGraphInner>>);

/// The inner graph type of [`NavGraph`].
pub type NavGraphInner = petgraph::graphmap::DiGraphMap<NavNode, NavGraphEdge>;

/// The type of nodes in the map navigation graph.
///
/// This is merely a wrapper around [`UVec2`] to add an [`Ord`] implementation.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Deref, DerefMut)]
pub struct NavNode(pub UVec2);
impl NavNode {
    pub fn right(&self) -> NavNode {
        NavNode(self.0 + uvec2(1, 0))
    }
    pub fn above(&self) -> NavNode {
        NavNode(self.0 + uvec2(0, 1))
    }
    pub fn left(&self) -> Option<NavNode> {
        if self.0.x > 0 {
            Some(NavNode(self.0 - uvec2(1, 0)))
        } else {
            None
        }
    }
    pub fn below(&self) -> Option<NavNode> {
        if self.0.y > 0 {
            Some(NavNode(self.0 - uvec2(0, 1)))
        } else {
            None
        }
    }
    pub fn below_left(&self) -> Option<NavNode> {
        self.left().and_then(|x| x.below())
    }
    pub fn below_right(&self) -> Option<NavNode> {
        self.below().map(|x| x.right())
    }
    pub fn above_left(&self) -> Option<NavNode> {
        self.left().map(|x| x.above())
    }
    pub fn above_right(&self) -> NavNode {
        self.right().above()
    }
}
impl From<UVec2> for NavNode {
    fn from(v: UVec2) -> Self {
        Self(v)
    }
}
impl From<NavNode> for UVec2 {
    fn from(v: NavNode) -> Self {
        v.0
    }
}
impl std::cmp::Ord for NavNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl std::cmp::PartialOrd for NavNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let xcmp = self.0.x.cmp(&other.0.x);
        Some(if xcmp == std::cmp::Ordering::Equal {
            self.0.y.cmp(&other.0.y)
        } else {
            xcmp
        })
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
    mut entities: ResMut<Entities>,
    mut clear_color: ResMut<ClearColor>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut map_spawned: ResMut<MapSpawned>,
    mut tiles: CompMut<Tile>,
    mut tile_layers: CompMut<TileLayer>,
    mut transforms: CompMut<Transform>,
    mut element_handles: CompMut<ElementHandle>,
    mut tile_collisions: CompMut<TileCollision>,
    mut parallax_bg_sprites: CompMut<ParallaxBackgroundSprite>,
    mut sprites: CompMut<Sprite>,
    mut nav_graph: ResMut<NavGraph>,
) {
    if map_spawned.0 {
        return;
    }
    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };
    map_spawned.0 = true;
    **clear_color = map.background_color.0;

    // Load the navigation graph
    nav_graph.0 = Some(create_nav_graph(map));

    // Spawn parallax backgrounds
    for layer in &map.background.layers {
        for i in -1..=1 {
            let ent = entities.create();
            sprites.insert(
                ent,
                Sprite {
                    image: layer.image.clone(),
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
    for (i, layer) in map.layers.iter().enumerate() {
        let layer_z = -900.0 + i as f32;
        match &layer.kind {
            MapLayerKind::Tile(tile_layer_meta) => {
                let mut tile_layer = TileLayer::new(
                    map.grid_size,
                    map.tile_size,
                    tile_layer_meta.tilemap.clone(),
                );

                for tile_meta in &tile_layer_meta.tiles {
                    let tile_ent = entities.create();
                    tile_layer.set(tile_meta.pos, Some(tile_ent));
                    tiles.insert(
                        tile_ent,
                        Tile {
                            idx: tile_meta.idx as usize,
                            ..default()
                        },
                    );
                    tile_collisions.insert(
                        tile_ent,
                        if tile_meta.jump_through {
                            TileCollision::JUMP_THROUGH
                        } else {
                            TileCollision::SOLID
                        },
                    );
                }
                let layer_ent = entities.create();
                tile_layers.insert(layer_ent, tile_layer);
                transforms.insert(
                    layer_ent,
                    Transform::from_translation(Vec3::new(0.0, 0.0, layer_z)),
                );
            }
            MapLayerKind::Element(element_layer_meta) => {
                for element_meta in &element_layer_meta.elements {
                    let element_ent = entities.create();

                    transforms.insert(
                        element_ent,
                        Transform::from_translation(element_meta.pos.extend(layer_z)),
                    );
                    element_handles
                        .insert(element_ent, ElementHandle(element_meta.element.clone()));
                }
            }
        }
    }
}

fn handle_out_of_bounds_players_and_items(
    entities: Res<Entities>,
    mut transforms: CompMut<Transform>,
    player_indexes: Comp<PlayerIdx>,
    map_handle: Res<MapHandle>,
    map_assets: BevyAssets<MapMeta>,
    mut player_events: ResMut<PlayerEvents>,
    map_respawn_points: Comp<MapRespawnPoint>,
) {
    const KILL_ZONE_BORDER: f32 = 500.0;
    let Some(map) = map_assets.get(&map_handle.get_bevy_handle()) else {
        return;
    };

    let map_width = map.grid_size.x as f32 * map.tile_size.x;
    let left_kill_zone = -KILL_ZONE_BORDER;
    let right_kill_zone = map_width + KILL_ZONE_BORDER;
    let bottom_kill_zone = -KILL_ZONE_BORDER;

    // Kill out of bounds players
    for (player_ent, (_player_idx, transform)) in entities.iter_with((&player_indexes, &transforms))
    {
        let pos = transform.translation;

        if pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone {
            player_events.kill(player_ent);
        }
    }

    // Reset out of bound item positions
    for (_ent, (respawn_point, transform)) in
        entities.iter_with((&map_respawn_points, &mut transforms))
    {
        let pos = transform.translation;

        if pos.x < left_kill_zone || pos.x > right_kill_zone || pos.y < bottom_kill_zone {
            transform.translation = respawn_point.0;
        }
    }
}

/// Helper method to create a navigation graph from the map metadata.
fn create_nav_graph(meta: &MapMeta) -> Arc<NavGraphInner> {
    // Load the navigation graph
    let mut graph = NavGraphInner::default();

    // Initialize set of traversable tiles, assuming all tiles are traversable
    let mut semi_solids = HashSet::default();
    for x in 0..meta.grid_size.x {
        for y in 0..meta.grid_size.y {
            graph.add_node(NavNode(uvec2(x, y)));
        }
    }
    // Find all solid tiles and remove them from the traversable tiles list
    for layer in &meta.layers {
        if let MapLayerKind::Tile(layer) = &layer.kind {
            for tile in &layer.tiles {
                if tile.jump_through {
                    semi_solids.insert(NavNode(tile.pos));
                } else {
                    graph.remove_node(NavNode(tile.pos));
                }
            }
        }
    }

    // Calculate possible movements from every node
    macro_rules! is_solid {
        ($node:ident) => {
            !graph.contains_node($node) || semi_solids.contains(&$node)
        };
    }
    for node in graph.nodes().collect::<Vec<_>>() {
        // Fall straight down
        if let Some(below) = node.below() {
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
                                PlayerControl {
                                    move_direction: vec2(0.0, -1.0),
                                    jump_pressed: true,
                                    ..default()
                                },
                                PlayerControl::default(),
                                PlayerControl::default(),
                            ]
                            .into(),
                            distance: 1.0,
                        },
                    );
                } else {
                    graph.add_edge(
                        node,
                        below,
                        NavGraphEdge {
                            inputs: [PlayerControl::default()].into(),
                            distance: 1.0,
                        },
                    );
                }
            }
        }
        // Fall diagonally down right
        if let Some(below_right) = node.below_right() {
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
                            distance: 1.0,
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
                            distance: 1.41,
                        },
                    );
                }
            }
        }
        // Fall diagonally down left
        if let Some(left) = node.below_left() {
            if graph.contains_node(left) {
                if semi_solids.contains(&left) {
                    graph.add_edge(
                        node,
                        left,
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
                            distance: 1.0,
                        },
                    );
                } else {
                    graph.add_edge(
                        node,
                        left,
                        NavGraphEdge {
                            inputs: [PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                ..default()
                            }]
                            .into(),
                            distance: 1.41,
                        },
                    );
                }
            }
        }

        // walk left or right along the ground
        let has_ground = node.below().map(|x| is_solid!(x)).unwrap_or_default()
            || node.below_left().map(|x| is_solid!(x)).unwrap_or_default()
            || node.below_right().map(|x| is_solid!(x)).unwrap_or_default();
        if has_ground {
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
                        distance: 1.0,
                    },
                );
            }

            if let Some(left) = node.left() {
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
                            distance: 1.0,
                        },
                    );
                }
            }

            let above1 = node.above();
            let above2 = above1.above();
            let above2l = above2.left();
            let above2r = above2.right();
            let above2l2 = above2l.and_then(|x| x.left());
            let above2r2 = above2r.right();
            if graph.contains_node(node.above()) && graph.contains_node(node.above().above()) {
                // Jump staight up
                graph.add_edge(
                    node,
                    node.above().above(),
                    NavGraphEdge {
                        inputs: [PlayerControl {
                            jump_just_pressed: true,
                            jump_pressed: true,
                            ..default()
                        }]
                        .into(),
                        distance: 2.0,
                    },
                );

                // Jump up and right
                if graph.contains_node(above2r) {
                    graph.add_edge(
                        node,
                        node.above().above(),
                        NavGraphEdge {
                            inputs: [PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            }]
                            .into(),
                            distance: 2.23,
                        },
                    );
                }
                // Jump up and right 2
                if graph.contains_node(above2r2) {
                    graph.add_edge(
                        node,
                        node.above().above(),
                        NavGraphEdge {
                            inputs: [PlayerControl {
                                move_direction: vec2(1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            }]
                            .into(),
                            distance: 2.82,
                        },
                    );
                }

                // Jump up and left
                if above2l.map(|x| graph.contains_node(x)).unwrap_or_default() {
                    graph.add_edge(
                        node,
                        node.above().above(),
                        NavGraphEdge {
                            inputs: [PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            }]
                            .into(),
                            distance: 2.23,
                        },
                    );
                }
                // jump up and left 2
                if above2l2.map(|x| graph.contains_node(x)).unwrap_or_default() {
                    graph.add_edge(
                        node,
                        node.above().above(),
                        NavGraphEdge {
                            inputs: [PlayerControl {
                                move_direction: vec2(-1.0, 0.0),
                                jump_just_pressed: true,
                                jump_pressed: true,
                                ..default()
                            }]
                            .into(),
                            distance: 2.82,
                        },
                    );
                }
            }
        }
    }

    Arc::new(graph)
}
