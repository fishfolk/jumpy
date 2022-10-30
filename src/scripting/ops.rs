use bevy_mod_js_scripting::OpMap;

pub mod asset;
pub mod net;
// mod commands;
pub mod entity;
pub mod map;
pub mod player;
pub mod script_info;

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
        "jumpy_asset_get_absolute_path",
        Box::new(asset::AssetGetAbsolutePath),
    );
    ops.insert(
        "jumpy_script_info_get",
        Box::new(script_info::ScriptInfoGet),
    );
    ops.insert("entity_ref_to_js", Box::new(entity::EntityRefToJs));
    ops.insert("entity_ref_from_js", Box::new(entity::EntityRefFromJs));
    ops.insert("jumpy_net_info_get", Box::new(net::NetInfoGet));
    ops.insert("jumpy_player_kill", Box::new(player::PlayerKill));

    // ops.insert(
    //     "jumpy_net_commands_spawn",
    //     Box::new(commands::NetCommandsSpawn),
    // );
    // ops.insert(
    //     "jumpy_net_commands_insert",
    //     Box::new(commands::NetCommandsInsert),
    // );

    ops
}
