use crate::prelude::*;
use bevy_mod_js_scripting::{JsRuntimeConfig, JsScriptingPlugin};

mod ops;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let custom_ops = ops::get_ops();

        app.register_type::<Time>()
            .insert_non_send_resource(JsRuntimeConfig { custom_ops })
            .add_plugin(JsScriptingPlugin {
                script_stages: [
                    (FixedUpdateStage::First.as_label(), "first".to_string()),
                    (
                        FixedUpdateStage::PreUpdate.as_label(),
                        "preUpdate".to_string(),
                    ),
                    (FixedUpdateStage::Update.as_label(), "update".to_string()),
                    (
                        FixedUpdateStage::PostUpdate.as_label(),
                        "postUpdate".to_string(),
                    ),
                    (FixedUpdateStage::Last.as_label(), "last".to_string()),
                ]
                .into_iter()
                .collect(),
            });
    }
}
