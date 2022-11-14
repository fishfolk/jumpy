use bevy_mod_js_scripting::JsRuntimeOp;

pub struct RollbackHooks;

impl JsRuntimeOp for RollbackHooks {
    fn js(&self) -> Option<&'static str> {
        Some(include_str!("./rollback/rollback_hooks.js"))
    }
}
