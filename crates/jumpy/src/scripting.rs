use crate::prelude::*;
use bevy_mod_js_scripting::{JsRuntimeConfig, JsScriptingPlugin};

mod ops;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let custom_ops = ops::get_ops();

        app.insert_non_send_resource(JsRuntimeConfig { custom_ops })
            .add_plugin(JsScriptingPlugin);
    }
}
