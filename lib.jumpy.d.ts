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

/**
 * TODO: These are functions that need to be moved to a different namespace, or built into
 * bevy_mod_js_scripting.
 *
 * Sorry for the weird naming!
 */
declare namespace WorldTemp {
  function spawn(): Entity;
  function despawnRecursive(entity: Entity): void;
}

// All handles have the same type, so just alias here
type HandleJsScript = HandleImage;

declare interface ScriptInfo {
  path: string;
  handle: HandleJsScript;
  handle_id_hash: string;
}

declare namespace Script {
  function getInfo(): ScriptInfo;
  function state<T>(init?: T): T;
  function addEntityToList(listName: string, entity: Entity): void;
  function getEntityList(listName: string): Entity[];
  function entityListContains(listName: string, entity: Entity): boolean;
  function removeEntityFromList(listName: string, entity: Entity): void;
  function clearEntityList(listName: string): void;
  function entityStates(): object;
  function getEntityState<T>(entity: Entity, init?: T): T;
  function setEntityState<T>(entity: Entity, value: T): void;
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
  player_idx: usize;
  player_count: usize;
}

declare namespace NetInfo {
  function get(): NetInfo;
}

type PlayerKillEvent = {
  player: Entity;
  velocity: Vec2;
  position: Vec3;
};
declare namespace Player {
  function kill(entity: Entity): void;
  function despawn(entity: Entity): void;
  function killEvents(): PlayerKillEvent[];
  function getInventory(player: Entity): Entity | null;
  function setInventory(player: Entity, item: Entity): void;
  function useItem(player: Entity): void;
}
declare namespace Random {
  function gen(): number;
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
declare const PlayerKilled: BevyType<unknown>;

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
type ItemGrabbed = {
  player: Entity;
};
declare const ItemGrabbed: BevyType<ItemGrabbed>;
type ItemDropped = {
  player: Entity;
};
declare const ItemDropped: BevyType<ItemDropped>;
type ItemUsed = {
  player: Entity;
};
declare const ItemUsed: BevyType<ItemUsed>;


type DamageRegion = {
  size: Vec2;
};
declare const DamageRegion: BevyType<DamageRegion>;
declare const DamageRegionOwner: BevyType<[Entity]>;

type Lifetime = {
  lifetime: f32;
  age: f32;
};
declare const Lifetime: BevyType<Lifetime>;

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
  is_spawning: boolean;
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
  index: usize;
  start: usize;
  end: usize;
  atlas: HandleTextureAtlas;
  flip_x: boolean;
  flip_y: boolean;
  repeat: boolean;
  fps: f32;
};
declare const AnimatedSprite: BevyType<AnimatedSprite>;
