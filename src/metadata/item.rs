use super::*;

#[derive(Component, HasLoadProgress, TypeUuid, Deserialize, Serialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
#[uuid = "24d6dfb1-9033-4d97-9021-021f752f77ef"]
pub struct ItemMeta {}
