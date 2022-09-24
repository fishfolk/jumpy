use bevy::reflect::TypeUuid;
use bevy_has_load_progress::HasLoadProgress;
use bevy_mod_js_scripting::JsScript;

use crate::prelude::*;

mod localization;

#[derive(HasLoadProgress, TypeUuid, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
#[uuid = "eb28180f-ef68-44a0-8479-a299a3cef66e"]
pub struct GameMeta {
    pub translations: localization::TranslationsMeta,
    pub camera_height: u32,

    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(skip)]
    pub script_handles: Vec<Handle<JsScript>>,
}
