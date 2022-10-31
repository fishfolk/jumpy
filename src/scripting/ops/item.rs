use std::sync::Mutex;

use crate::{
    item::{Item, ItemDropEvent, ItemGrabEvent},
    prelude::*,
};
use bevy::ecs::system::SystemState;
use bevy_mod_js_scripting::{serde_json, JsRuntimeOp, JsValueRef, OpContext};
use once_cell::sync::OnceCell;

pub struct ItemGrabEvents;
impl JsRuntimeOp for ItemGrabEvents {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Items) {
                globalThis.Items = {}
            }
            
            globalThis.Items.grabEvents = () => {
                return bevyModJsScriptingOpSync('jumpy_item_grab_events')
                    .map(x => globalThis.Value.wrapValueRef(x));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        _args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let script_path = ctx.script_info.path.to_str().expect("non-unicode path");

        type Param<'w, 's> = (
            Query<'w, 's, &'static Item>,
            EventReader<'w, 's, ItemGrabEvent>,
        );
        static STATE: OnceCell<Mutex<SystemState<Param>>> = OnceCell::new();
        let mut state = STATE
            .get_or_init(|| Mutex::new(SystemState::new(world)))
            .lock()
            .unwrap();

        let value_refs = ctx.op_state.get_mut().unwrap();

        let (items, mut grab_events) = state.get_mut(world);

        let events = grab_events
            .iter()
            .filter(|event| {
                items
                    .get(event.item)
                    .map(|item| item.script == script_path)
                    .unwrap_or(false)
            })
            .map(|x| JsValueRef::new_free(Box::new(x.clone()), value_refs))
            .collect::<Vec<_>>();

        Ok(serde_json::to_value(&events)?)
    }
}

pub struct ItemDropEvents;
impl JsRuntimeOp for ItemDropEvents {
    fn js(&self) -> Option<&'static str> {
        Some(
            r#"
            if (!globalThis.Items) {
                globalThis.Items = {}
            }
            
            globalThis.Items.dropEvents = () => {
                return bevyModJsScriptingOpSync('jumpy_item_drop_events')
                    .map(x => globalThis.Value.wrapValueRef(x));
            }
            "#,
        )
    }

    fn run(
        &self,
        ctx: OpContext,
        world: &mut World,
        _args: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let script_path = ctx.script_info.path.to_str().expect("non-unicode path");

        type Param<'w, 's> = (
            Query<'w, 's, &'static Item>,
            EventReader<'w, 's, ItemDropEvent>,
        );
        static STATE: OnceCell<Mutex<SystemState<Param>>> = OnceCell::new();
        let mut state = STATE
            .get_or_init(|| Mutex::new(SystemState::new(world)))
            .lock()
            .unwrap();

        let value_refs = ctx.op_state.get_mut().unwrap();

        let (items, mut grab_events) = state.get_mut(world);

        let events = grab_events
            .iter()
            .filter(|event| {
                items
                    .get(event.item)
                    .map(|item| item.script == script_path)
                    .unwrap_or(false)
            })
            .map(|x| JsValueRef::new_free(Box::new(x.clone()), value_refs))
            .collect::<Vec<_>>();

        Ok(serde_json::to_value(&events)?)
    }
}
