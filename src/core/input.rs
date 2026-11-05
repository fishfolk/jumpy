//! Player and editor input types.

use std::array;

use bones_framework::input::Controls;

use crate::{prelude::*, MAX_PLAYERS};

pub fn install(session: &mut SessionBuilder) {
    session.init_resource::<MatchInputs>();
}

/// The inputs for each player in this simulation frame.
#[derive(Clone, Debug, HasSchema)]
pub struct MatchInputs {
    pub players: [PlayerInput; MAX_PLAYERS as usize],
}
impl MatchInputs {
    pub fn get_control_source(&self, player_idx: usize) -> Option<ControlSource> {
        self.players.get(player_idx).unwrap().control_source
    }
}
impl Default for MatchInputs {
    fn default() -> Self {
        Self {
            players: array::from_fn(|_| default()),
        }
    }
}
impl Controls<'_, PlayerControl> for MatchInputs {
    fn get_control(&self, player_idx: usize) -> &PlayerControl {
        &self.players.get(player_idx).unwrap().control
    }

    fn get_control_mut(&mut self, player_idx: usize) -> &mut PlayerControl {
        &mut self.players.get_mut(player_idx).unwrap().control
    }
}

/// Player input, not just controls, but also other status that comes from the player, such as the
/// selected player and whether the player is actually active.
#[derive(Default, Clone, Debug, HasSchema)]
pub struct PlayerInput {
    /// Whether or not the player is present.
    pub active: bool,
    /// The selected player skin.
    pub selected_player: Handle<PlayerMeta>,
    /// The selected player hat.
    pub selected_hat: Option<Handle<HatMeta>>,
    /// The player control input
    pub control: PlayerControl,
    /// The editor inputs the player is making, if any.
    pub editor_input: Option<EditorInput>,
    /// If this is [`None`] it means the player is an AI, or remote player in networked game.
    pub control_source: Option<ControlSource>,

    /// Whether or not this is an AI player.
    pub is_ai: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocatedTileLayer {
    pub layer_index: u32,
    pub located_tiles: Vec<(UVec2, u32, TileCollisionKind)>,
}

#[derive(Clone, HasSchema, Default, Debug)]
pub struct ElementLayer {
    pub layer_index: u32,
    pub located_elements: Vec<(Vec2, Handle<ElementMeta>)>,
}

/// The editor inputs that a player may make.
#[derive(Clone, Debug)]
pub enum EditorInput {
    /// Spawn an element onto the map.
    SpawnElement {
        /// The handle to the element that is being spawned.
        handle: Handle<ElementMeta>,
        /// The translation to spawn the element with.
        translation: Vec2,
        /// The map layer index to spawn the element on.
        layer: u8,
    },
    MoveEntity {
        /// The entity to move.
        entity: Entity,
        /// The amount to move the entity.
        pos: Vec2,
    },
    DeleteEntity {
        /// The entity to delete.
        entity: Entity,
    },
    /// Create a new layer
    CreateLayer {
        /// The name of the layer.
        id: String,
    },
    /// Rename a map layer.
    RenameLayer {
        /// The index of the layer to rename.
        layer: u8,
        /// The new name of the layer.
        name: String,
    },
    DeleteLayer {
        layer: u8,
    },
    /// Move a layer up or down.
    MoveLayer {
        /// The layer to move
        layer: u8,
        /// Whether or not to move the layer down. If false, move the layer up.
        down: bool,
    },
    /// Update the tilemap of a layer.
    SetTilemap {
        /// The layer index of the layer to update.
        layer: u8,
        /// The handle to the tilemap to use or [`None`] to clear the tilemap.
        handle: Option<Handle<Atlas>>,
    },
    SetTile {
        /// The layer index of the layer to update
        layer: u8,
        /// The position of the tile to set
        pos: UVec2,
        /// The index in the tilemap to set the tile, or [`None`] to delete the tile.
        tilemap_tile_idx: Option<u32>,
        /// The tile collision kind
        collision: TileCollisionKind,
    },
    RenameMap {
        name: String,
    },
    RandomizeTiles {
        tile_layers: Vec<LocatedTileLayer>,
        element_layers: Vec<ElementLayer>,
        tile_size: Vec2,
    },
}
