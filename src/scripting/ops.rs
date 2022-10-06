use bevy_mod_js_scripting::OpMap;

mod asset;
mod map;

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert(
        "jumpy_element_get_spawned_entities",
        Box::new(map::ElementGetSpawnedEntities),
    );
    ops.insert(
        "jumpy_asset_get_absolute_path",
        Box::new(asset::AssetAbsolutePath),
    );

    ops
}
