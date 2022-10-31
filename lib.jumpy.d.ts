//
// Core Jumpy script ops and namespaces
//

declare namespace MapElement {
  function getSpawnedEntities(): Entity[];
  function mapReset(): boolean;
}

/** We've added a reflect function for hashing the HandleId to a JS Number */
interface HandleIdWithFuncs {
  hash(): string;
}

declare namespace Assets {
  function getHandleId(relative_path: string): HandleIdWithFuncs;
  function getAbsolutePath(relative_path: string): string;
}

// All handles have the same type, so just alias here
type HandleJsScript = HandleImage;

declare interface ScriptInfo {
  path: string;
  handle: HandleJsScript;
  handle_id_hash: string;
}

declare namespace ScriptInfo {
  function get(): ScriptInfo;
  function state<T>(init?: T): T;
}

declare namespace NetCommands {
  function insert(entity: Entity, component: any): void;
  function spawn(): Entity;
}

declare type JsEntity = {
  bits: number;
};

declare namespace EntityRef {
  function fromJs(js_ent: JsEntity): Entity;
  function toJs(ent: Entity): JsEntity;
}

interface NetInfo {
  is_client: boolean;
  is_server: boolean;
  player_idx: usize;
}

declare namespace NetInfo {
  function get(): NetInfo;
}

declare namespace Player {
  function kill(entity: Entity): void;
  function getInventory(player: Entity): Entity | null;
  function setInventory(player: Entity, item: Entity): void;
}

declare namespace CollisionWorld {
  function actorCollisions(entity: Entity): Entity[];
}

//
// Jumpy component types
//

type EntityName = [string];
declare const EntityName: BevyType<EntityName>;

type MapMeta = {
  name: string;
  grid_size: UVec2;
  tile_size: UVec2;
};
declare const MapMeta: BevyType<MapMeta>;
declare const GameCamera: BevyType<unknown>;

type PlayerIdx = [usize];
declare const PlayerIdx: BevyType<PlayerIdx>;

type PlayerState = { id: string; age: u64; previous_state: string };
declare const PlayerState: BevyType<PlayerState>;

type HandlePlayerMeta = HandleImage;
type PlayerControl = {
  move_direction: Vec2;
  jump_pressed: boolean;
  jump_just_pressed: boolean;
  shoot_pressed: boolean;
  shoot_just_pressed: boolean;
  grab_pressed: boolean;
  grab_just_pressed: boolean;
  slide_pressed: boolean;
  slide_just_pressed: boolean;
};
type PlayerInput = {
  active: boolean;
  selected_player: HandlePlayerMeta;
  control: PlayerControl;
  previous_control: PlayerControl;
};
type PlayerInputs = {
  players: PlayerInput[];
};
declare const PlayerInputs: BevyType<PlayerInputs>;

type Item = {
  script: string;
};
declare const Item: BevyType<Item>;

type KinematicBody = {
  offset: Vec2;
  size: Vec2;
  velocity: Vec2;
  is_on_ground: boolean;
  was_on_ground: boolean;
  has_mass: boolean;
  has_friction: boolean;
  bouncyness: f32;
  is_deactivated: boolean;
  gravity: f32;
  fall_through: boolean;
};
declare const KinematicBody: BevyType<KinematicBody>;

type AnimationBankSprite = {
  current_animation: string;
  flip_x: boolean;
  flip_y: boolean;
  animations: unknown;
};
declare const AnimationBankSprite: BevyType<AnimationBankSprite>;

type AnimatedSprite = {
  start: usize;
  end: usize;
  atlas: HandleTextureAtlas;
  flip_x: boolean;
  flip_y: boolean;
  repeat: boolean;
  fps: f32;
};
declare const AnimatedSprite: BevyType<AnimatedSprite>;
