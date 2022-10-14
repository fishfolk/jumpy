use std::hash::{Hash, Hasher};

use crate::prelude::*;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, OpContext};

#[derive(Serialize)]
struct JsScriptInfo {
    path: String,
    handle: JsValueRef,
    handle_id_hash: u64,
}

pub struct ScriptInfoGet;
impl JsRuntimeOp for ScriptInfoGet {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.ScriptInfo) {
                globalThis.ScriptInfo = {}
            }
            
            globalThis.ScriptInfo.get = () => {
                return bevyModJsScriptingOpSync('jumpy_script_info_get');
            }

            globalThis.ScriptInfo.state = (init) => {
                const scriptId = ScriptInfo.get().handle_id_hash;
                if (!globalThis.scriptState) globalThis.scriptState = {};
                if (!globalThis.scriptState[scriptId]) globalThis.scriptState[scriptId] = init || {}
                return globalThis.scriptState[scriptId];
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
        // The bit shift makes the hash fit within the safe integer range for a JavaScript number
        let hash = hasher.finish() >> 12;

        Ok(serde_json::to_value(&JsScriptInfo {
            path: ctx.script_info.path.to_string_lossy().into(),
            handle: JsValueRef::new_free(Box::new(ctx.script_info.handle.clone()), value_refs),
            handle_id_hash: hash,
        })?)
    }
}
