use crate::prelude::*;
use anyhow::Context;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, OpContext};

pub struct AssetAbsolutePath;
impl JsRuntimeOp for AssetAbsolutePath {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Assets) {
                globalThis.Assets = {}
            }
            
            globalThis.Assets.absolutePath = (path) => {
                return Value.wrapValueRef(bevyModJsScriptingOpSync('jumpy_asset_get_absolute_path', path));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        _world: &mut World,
        args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (relative_path,): (String,) = serde_json::from_value(args).context("Parse args")?;

        let absolute_path = ctx
            .script_info
            .path
            .parent()
            .unwrap()
            .join(relative_path)
            .to_str()
            .expect("Non-utf8 path")
            .to_owned();

        Ok(serde_json::to_value(&absolute_path)?)
    }
}
