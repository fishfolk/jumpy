use bevy_mod_js_scripting::{JsRuntimeOp, OpMap};

pub mod asset;
pub mod collision_world;
pub mod entity;
pub mod map;
pub mod net;
pub mod player;
pub mod script_info;

pub fn get_ops() -> OpMap {
    let mut ops = OpMap::default();

    ops.insert("_component_types_include", Box::new(ComponentTypesInclude));
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
    ops.insert(
        "jumpy_player_get_inventory",
        Box::new(player::PlayerGetInventory),
    );
    ops.insert(
        "jumpy_player_set_inventory",
        Box::new(player::PlayerSetInventory),
    );
    ops.insert(
        "jumpy_collision_world_actor_collisions",
        Box::new(collision_world::CollisionWorldActorCollisions),
    );

    ops
}

/// Op that includes the type name constants for components defined in Jumpy. A temporary fix until
/// they can be auto-generated by bevy_mod_js_scripting.
struct ComponentTypesInclude;
impl JsRuntimeOp for ComponentTypesInclude {
    fn js(&self) -> Option<&'static str> {
        Some(include_str!("../../lib.jumpy.js"))
    }
}
