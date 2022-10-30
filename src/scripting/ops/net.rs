use crate::{
    networking::{client::NetClient, proto::ClientMatchInfo, server::NetServer},
    prelude::*,
};
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, OpContext};

#[derive(Serialize)]
struct JsNetInfo {
    is_server: bool,
    is_client: bool,
    player_idx: usize,
}

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
        let is_server = world.contains_resource::<NetServer>();
        let is_client = world.contains_resource::<NetClient>();
        let match_info = world.get_resource::<ClientMatchInfo>();
        let player_idx = match_info.map(|info| info.player_idx).unwrap_or(0);

        Ok(serde_json::to_value(&JsNetInfo {
            is_server,
            is_client,
            player_idx,
        })?)
    }
}
