use std::{error::Error, sync::Arc};

use hecs::World;
use hv_cell::AtomicRefCell;
use hv_lua::{chunk, Lua, Value};
use macroquad::prelude::collections::storage;
use macroquad_platformer::Actor;
use tealr::mlu::TealData;
use tealr::TypeName;

use crate::Resources;

pub(crate) fn run_event(
    event_name: &'static str,
    world: Arc<AtomicRefCell<World>>,
) -> Result<(), Box<dyn Error>> {
    let res = storage::get_mut::<Resources>();
    let lua = &res.lua;
    let thread_name = format!("Event: {}", event_name);
    let chunk = chunk! {
        local world = $world
        local event_name = $event_name
        local events_to_run = events[event_name] or {}
        for _ , mod_config in ipairs(events_to_run) do
            require = mod_config.require
            event = mod_config.events[event_name]
            if type(event) == "function" then
                local isSuccess, err = pcall(event,world)
                if not isSuccess then
                    io.stderr:write("Error while calling: `",event_name, "` from mod: `",mod_config.mod_id,"` Error:\n",err,"\n")
                end
            end
        end
        require = nil
    };
    lua.load(chunk).set_name(&thread_name)?.exec()?;
    Ok(())
}
use core::lua::CopyComponent;

use core::create_type_component_container;
create_type_component_container!(
    TypeComponentContainer with
    I32 of CopyComponent<i32>,
    Bool of CopyComponent<bool>,
);

pub(crate) fn register_types(lua: &Lua) -> Result<Value, Box<dyn Error>> {
    use hv_lua::ToLua;
    Ok(TypeComponentContainer::new(lua)?.to_lua(lua)?)
}

use hv_lua as mlua;

#[derive(Clone, Copy, tealr::MluaUserData)]
pub struct ActorLua(Actor);
impl TypeName for ActorLua {
    fn get_type_parts() -> std::borrow::Cow<'static, [tealr::NamePart]> {
        tealr::new_type!(Actor, External)
    }
}
impl TealData for ActorLua {}

impl From<ActorLua> for Actor {
    fn from(a: ActorLua) -> Self {
        a.0
    }
}
impl From<Actor> for ActorLua {
    fn from(a: Actor) -> Self {
        Self(a)
    }
}
