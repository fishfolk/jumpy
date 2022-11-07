use crate::{networking::proto::ClientMatchInfo, prelude::*};
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, OpContext};

pub struct NetInfoGet;
impl JsRuntimeOp for NetInfoGet {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.NetInfo) {
                globalThis.NetInfo = {}
            }
            
            globalThis.NetInfo.get = () => {
                return bevyModJsScriptingOpSync('jumpy_net_info_get');
            }
            "#,
        )
    }

    fn run(
        &self,
        _ctx: OpContext,
        world: &mut World,
        _args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let match_info = world.get_resource::<ClientMatchInfo>();
        Ok(serde_json::to_value(&match_info)?)
    }
}
