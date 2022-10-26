use bevy_mod_js_scripting::OpMap;

mod asset;
mod commands;
mod entity;
mod map;
mod script_info;

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert(
        "jumpy_element_get_spawned_entities",
        Box::new(map::ElementGetSpawnedEntities),
    );
    ops.insert(
        "jumpy_asset_get_handle_id",
        Box::new(asset::AssetGetHandleId),
    );
    ops.insert(
        "jumpy_script_info_get",
        Box::new(script_info::ScriptInfoGet),
    );
    ops.insert("entity_ref_to_js", Box::new(entity::EntityRefToJs));
    ops.insert("entity_ref_from_js", Box::new(entity::EntityRefFromJs));
    ops.insert(
        "jumpy_net_commands_spawn",
        Box::new(commands::NetCommandsSpawn),
    );
    ops.insert(
        "jumpy_net_commands_insert",
        Box::new(commands::NetCommandsInsert),
    );

    ops
}
