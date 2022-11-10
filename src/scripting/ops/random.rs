use crate::{prelude::*, random::GlobalRng};
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, OpContext};

pub struct Random;
impl JsRuntimeOp for Random {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Random) {
                globalThis.Random = {}
            }
            
            globalThis.Random.gen = () => {
                return bevyModJsScriptingOpSync('jumpy_random');
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
        let rng = world.resource::<GlobalRng>();
        Ok(serde_json::to_value(rng.f32())?)
    }
}
