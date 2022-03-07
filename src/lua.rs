use std::{error::Error, os::unix::prelude::OsStrExt, path::Path};

use hv_lua::{chunk, Function, Lua};
use macroquad::prelude::collections::storage;

use crate::Resources;

const LUA_ENTRY: &str = "main";

pub(crate) fn init_lua(mod_dir: &Path) -> Result<Lua, Box<dyn Error>> {
    let lua = Lua::new();
    let dir = mod_dir.as_os_str().as_bytes().to_owned();
    {
        let globals = lua.globals();
        //a table containing all the mods
        globals.set("mods", lua.create_table()?)?;

        //we don't want users to simply use `require` to load their files as that will either result in conflicts or annoying path names
        //to drive this point home, I rename the `require` function here and add 2 other functions to load mod files
        let req = globals.get::<_, Function>("require")?;
        globals.set("require_lib", req)?;
        globals.set("require", hv_lua::Nil)?;

        //this function allows people to load a file from arbitrary mods
        //this allows "library only" mods to exist.
        //there will also be a function that loads from the current mod. However, that one needs to be defined when loading the mod
        let load_any_mod_file = lua.create_function(
            move |lua, (from_mod, path): (hv_lua::String, hv_lua::String)| {
                //TODO: a way to go from `mod id` to `mod folder`
                let req = lua.globals().get::<_, Function>("require_lib")?;
                let mut full_path = Vec::new();
                full_path.extend_from_slice(&dir);
                full_path.extend_from_slice(b".");
                full_path.extend_from_slice(from_mod.as_bytes());
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

pub(crate) fn load_lua<P: AsRef<[u8]>>(mod_name: P, lua: &Lua) -> Result<(), Box<dyn Error>> {
    //I want to use a lua string but... those get turned into `nil` for some reason?
    //probably a bug in mlua/hv-lua that needs to be tackled
    let name = String::from_utf8_lossy(mod_name.as_ref());
    //a nice copy so the error message can easily specify what mod it failed to load
    let name2 = String::from_utf8_lossy(mod_name.as_ref());
    let entry = LUA_ENTRY.to_string();
    let chunk = chunk! {
        local name = $name
        local entry = $entry

        function require(load_path)
            return require_from(name , load_path)
        end
        local function init_mod()
            return require(entry)
        end
        local mod_res = {
            mod = init_mod()
        }
        mod_res["require"] = require
        mods[name] = mod_res
        require = nil
    };
    lua.load(chunk)
        .set_name(&format!("Load mod: {:?}", name2))?
        .exec()?;

    Ok(())
}

pub(crate) fn run_event(event_name: &'static str) -> Result<(), Box<dyn Error>> {
    let res = storage::get_mut::<Resources>();
    let lua = &res.lua;
    let thread_name = format!("Event: {}", event_name);
    let chunk = chunk! {
        local event_name = $event_name
        for mod_name, mod in pairs(mods) do
            local event = mod.mod[event_name]
            if event and type(event) == "function" then
                require = mod.require
                event()
            end
        end
        require = nil
    };
    lua.load(chunk).set_name(&thread_name)?.exec()?;
    Ok(())
}
