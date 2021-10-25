use serde::{Deserialize, Serialize};

use crate::components::{AnimationParams, AnimationPlayer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    pub animation: AnimationParams,
}

pub struct Equipment {
    pub id: String,
    pub sprite_animation: AnimationPlayer,
    pub uses: Option<u32>,
    pub use_cnt: u32,
}

impl Equipment {
    pub fn new(id: &str, params: EquipmentParams) -> Self {
        let sprite_animation = AnimationPlayer::new(params.animation);

        Equipment {
            id: id.to_string(),
            sprite_animation,
            uses: params.uses,
            use_cnt: 0,
        }
    }
}
