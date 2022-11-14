use std::hash::{Hash, Hasher};

use crate::{prelude::*, utils::cache_str};
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, OpContext};
use normalize_path::NormalizePath;

#[derive(Serialize)]
struct JsScriptInfo {
    path: String,
    handle: JsValueRef,
    handle_id_hash: String,
}

pub struct ScriptGetInfo;
impl JsRuntimeOp for ScriptGetInfo {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            const cloneObj = x => JSON.parse(JSON.stringify(x));
            if (!globalThis.Script) {
                globalThis.Script = {}
            }
            
            globalThis.Script.getInfo = () => {
                return bevyModJsScriptingOpSync('jumpy_script_get_info');
            }

            globalThis.Script.state = (init) => {
                const scriptId = Script.getInfo().path;
                if (!globalThis.jsState) globalThis.jsState = {};
                if (!globalThis.jsState.script) globalThis.jsState.script = {};
                if (!globalThis.jsState.script[scriptId]) globalThis.jsState.script[scriptId] = cloneObj(init) || {};
                return globalThis.jsState.script[scriptId];
            }

            globalThis.Script.entityStates = () => {
                if (!globalThis.jsState) globalThis.jsState = {};
                if (!globalThis.jsState.entity) globalThis.jsState.script = {};
                if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.script[scriptId] = {};
                return globalThis.jsState.entity[scriptId];
            }

            globalThis.Script.getEntityState = (entity, init) => {
                const jsEntity = EntityRef.toJs(entity);
                const entityKey = JSON.stringify(jsEntity);
                const scriptId = Script.getInfo().path;
                if (!globalThis.jsState.entity) globalThis.jsState.entity = {};
                if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.entity[scriptId] = {};
                if (!globalThis.jsState.entity[scriptId][entityKey]) globalThis.jsState.entity[scriptId][entityKey] = cloneObj(init) || {};
                return globalThis.jsState.entity[scriptId][entityKey];
            }
            globalThis.Script.setEntityState = (entity, state) => {
                const jsEntity = EntityRef.toJs(entity);
                const entityKey = JSON.stringify(jsEntity);
                const scriptId = Script.getInfo().path;
                if (!globalThis.jsState.entity) globalThis.jsState.entity = {};
                if (!globalThis.jsState.entity[scriptId]) globalThis.jsState.entity[scriptId] = {};
                globalThis.jsState.entity[scriptId][entityKey] = state;
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        _world: &mut World,
        _args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let value_refs = ctx.op_state.entry().or_insert_with(default);

        let mut hasher = fnv::FnvHasher::default();
        ctx.script_info.handle.id.hash(&mut hasher);
        let hash = base64::encode(hasher.finish().to_le_bytes());
        cache_str(&hash);

        let path = ctx.script_info.path.normalize();
        let path_string = path.to_str().expect("Non-unicode path").to_owned();

        Ok(serde_json::to_value(&JsScriptInfo {
            path: path_string,
            handle: JsValueRef::new_free(Box::new(ctx.script_info.handle.clone()), value_refs),
            handle_id_hash: hash,
        })?)
    }
}
