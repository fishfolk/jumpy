use crate::{prelude::*, MAX_PLAYERS};

pub fn install(session: &mut CoreSession) {
    session.world.init_resource::<PlayerInputs>();
}

/// The inputs for each player in this simulation frame.
#[derive(Clone, Debug, TypeUlid)]
#[ulid = "01GP233N26N8DQAAS1WDGYM14X"]
pub struct PlayerInputs {
    pub players: Vec<PlayerInput>,
}

impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            players: vec![default(); MAX_PLAYERS],
            // has_updated: false,
        }
    }
}

/// Player input, not just controls, but also other status that comes from the player, such as the
/// selected player and whether the player is actually active.
#[derive(Default, Clone, Debug, TypeUlid)]
#[ulid = "01GP2356AYJ5NW8GJ3WF0TCCY3"]
pub struct PlayerInput {
    /// The player is currently "connected" and actively providing input.
    pub active: bool,
    /// This may be a null handle if a player hasn't been selected yet
    pub selected_player: Handle<PlayerMeta>,
    /// The player control input
    pub control: PlayerControl,
    /// The player control input from the last fixed update
    pub previous_control: PlayerControl,
    /// The editor inputs the player is making, if any.
    pub editor_input: Option<EditorInput>,
    /// Whether or not this is an AI player.
    pub is_ai: bool,
}

/// Player control input state
#[derive(Default, Clone, Debug)]
#[repr(C)]
pub struct PlayerControl {
    pub move_direction: Vec2,
    pub just_moved: bool,
    pub moving: bool,

    pub jump_pressed: bool,
    pub jump_just_pressed: bool,

    pub shoot_pressed: bool,
    pub shoot_just_pressed: bool,

    pub grab_pressed: bool,
    pub grab_just_pressed: bool,

    pub slide_pressed: bool,
    pub slide_just_pressed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileLayer {
    pub layer_index: usize,
    pub located_tiles: Vec<(UVec2, u32, TileCollisionKind)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElementLayer {
    pub layer_index: usize,
    pub located_elements: Vec<(Vec2, Handle<ElementMeta>)>,
}

/// The editor inputs that a player may make.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        tilemap_tile_idx: Option<usize>,
        /// The tile collision kind
        collision: TileCollisionKind,
    },
    RenameMap {
        name: String,
    },
    RandomizeTiles {
        tile_layers: Vec<TileLayer>,
        element_layers: Vec<ElementLayer>,
        tile_size: Vec2,
    },
}
