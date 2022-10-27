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
}

declare namespace NetInfo {
  function get(): NetInfo;
}
