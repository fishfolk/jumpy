use core::test::{BoolComponent, I32Component};
use std::{error::Error, os::unix::prelude::OsStrExt, path::Path, sync::Arc};

use hecs::World;
use hv_cell::AtomicRefCell;
use hv_lua::{chunk, hv::types, Function, Lua, Table};
use macroquad::prelude::collections::storage;

use crate::Resources;

const LUA_ENTRY: &str = "main";

pub(crate) fn init_lua(mod_dir: &Path) -> Result<Lua, Box<dyn Error>> {
    let lua = Lua::new();
    let hv = types(&lua)?;
    let i32_ty = lua.create_userdata_type::<I32Component>()?;
    let bool_ty = lua.create_userdata_type::<BoolComponent>()?;
    let dir = mod_dir.as_os_str().as_bytes().to_owned();
    {
        let globals = lua.globals();
        //a table containing all the mods
        globals.set("mods", lua.create_table()?)?;
        //this table is used to have quick access to every event that needs to run.
        globals.set("events", lua.create_table()?)?;

        //we don't want users to simply use `require` to load their files as that will either result in conflicts or annoying path names
        //to drive this point home, I rename the `require` function here and add 2 other functions to load mod files
        let req = globals.get::<_, Function>("require")?;
        globals.set("require_lib", req)?;
        globals.set("require", hv_lua::Nil)?;

        globals.set("hv", hv)?;
        globals.set("I32", i32_ty)?;
        globals.set("Bool", bool_ty)?;

        //this function allows people to load a file from arbitrary mods
        //this allows "library only" mods to exist.
        //there will also be a function that loads from the current mod. However, that one needs to be defined when loading the mod
        let load_any_mod_file = lua.create_function(
            move |lua, (from_mod, path): (hv_lua::String, hv_lua::String)| {
                //TODO: a way to go from `mod id` to `mod folder`
                let globals = lua.globals();
                let req = globals.get::<_, Function>("require_lib")?;
                let mod_folder = globals
                    .get::<_, Table>("mods")?
                    .get::<_, Table>(from_mod)?
                    .get::<_, hv_lua::String>("dir_name")?;

                let mut full_path = Vec::new();
                full_path.extend_from_slice(&dir);
                full_path.extend_from_slice(b".");
                full_path.extend_from_slice(mod_folder.as_bytes());
                full_path.extend_from_slice(b".");
                full_path.extend_from_slice(path.as_bytes());
                let full_path = lua.create_string(&full_path)?;
                req.call::<_, hv_lua::Value>(full_path)
            },
        )?;
        globals.set("require_from", load_any_mod_file)?;
    }
    Ok(lua)
}

pub(crate) fn load_lua<P: AsRef<[u8]>>(
    mod_id: String,
    mod_folder: P,
    lua: &Lua,
) -> Result<(), Box<dyn Error>> {
    let name = mod_id;
    let name_to_transfer = name.clone();
    //ideally, we use a lua string but... those don't get transferred properly for some reason....
    let dir = String::from_utf8_lossy(mod_folder.as_ref());
    let entry = LUA_ENTRY.to_string();
    let chunk = chunk! {
        local name = $name_to_transfer
        local dir = $dir
        local entry = $entry

        function require(load_path)
            return require_from(name , load_path)
        end
        local function init_mod()
            return require(entry)
        end
        local mod_config = {
            dir_name = dir,
            require = require,
            mod_id = name
        }
        mods[name] = mod_config

        local mod_events = init_mod()
        mod_config["events"] = mod_events
        if type(mod_events) == "table" then
            for k, _ in pairs(mod_events) do
                local event_list = events[k] or {}
                table.insert(event_list, mod_config)
                events[k] = event_list
            end
        end
        require = nil
    };
    lua.load(chunk)
        .set_name(&format!("Load mod: {:?}", name))?
        .exec()?;

    Ok(())
}

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
