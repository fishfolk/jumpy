declare namespace MapElement {
  function getSpawnedEntities(): Entity[];
  function mapReset(): boolean;
}

declare namespace Assets {
  function getHandleId(relative_path: string): HandleId;
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

declare type JsEntity = {
  bits: number;
};

declare namespace EntityRef {
  function fromJs(js_ent: JsEntity): Entity;
  function toJs(ent: Entity): JsEntity;
}
