use bevy_mod_js_scripting::OpMap;

mod asset;
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

    ops
}
