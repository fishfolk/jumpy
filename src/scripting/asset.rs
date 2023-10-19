use crate::prelude::*;

/// A Lua script asset.
#[derive(HasSchema)]
#[schema(no_clone, no_default)]
#[type_data(asset_loader("lua", LuaScriptLoader))]
pub struct LuaScript {
    pub source: String,
}

struct LuaScriptLoader;

impl AssetLoader for LuaScriptLoader {
    fn load(
        &self,
        _ctx: AssetLoadCtx,
        bytes: &[u8],
    ) -> futures::future::Boxed<anyhow::Result<SchemaBox>> {
        let bytes = bytes.to_vec();
        Box::pin(async move {
            let script = LuaScript {
                source: String::from_utf8(bytes)?,
            };
            Ok(SchemaBox::new(script))
        })
    }
}
