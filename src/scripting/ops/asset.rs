use std::path::PathBuf;

use crate::prelude::*;

use anyhow::Context;
use bevy::asset::HandleId;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, OpContext};
use normalize_path::NormalizePath;

pub struct AssetGetHandleId;
impl JsRuntimeOp for AssetGetHandleId {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Assets) {
                globalThis.Assets = {}
            }
            
            globalThis.Assets.getHandleId = (path) => {
                return Value.wrapValueRef(bevyModJsScriptingOpSync('jumpy_asset_get_handle_id', path));
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

        let reflect_refs = ctx.op_state.get_mut().unwrap();

        let path = &ctx.script_info.path;
        let absolute_path = if relative_path.starts_with('/') {
            PathBuf::from(relative_path)
        } else if relative_path.starts_with("./") {
            path.parent()
                .unwrap()
                .join(relative_path.strip_prefix("./").unwrap())
        } else {
            path.parent().unwrap().join(relative_path)
        };
        let absolute_path = absolute_path.normalize();
        let path_str = absolute_path.to_str().expect("Non-unicode-path");

        let handle_id: HandleId = path_str.into();
        let value_ref = JsValueRef::new_free(Box::new(handle_id), reflect_refs);

        Ok(serde_json::to_value(&value_ref)?)
    }
}

pub struct AssetGetAbsolutePath;
impl JsRuntimeOp for AssetGetAbsolutePath {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Assets) {
                globalThis.Assets = {}
            }
            
            globalThis.Assets.getAbsolutePath = (path) => {
                return bevyModJsScriptingOpSync('jumpy_asset_get_absolute_path', path);
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

        let path = &ctx.script_info.path;
        let absolute_path = if relative_path.starts_with('/') {
            PathBuf::from(relative_path)
        } else if relative_path.starts_with("./") {
            path.parent()
                .unwrap()
                .join(relative_path.strip_prefix("./").unwrap())
        } else {
            path.parent().unwrap().join(relative_path)
        };
        let absolute_path = absolute_path.normalize();
        let path_str = absolute_path.to_str().expect("Non-unicode-path");

        Ok(serde_json::to_value(path_str)?)
    }
}
