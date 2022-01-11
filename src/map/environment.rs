use serde::{Deserialize, Serialize};

use crate::AnimatedSpriteMetadata;

#[derive(Clone, Serialize, Deserialize)]
pub struct EnvironmentItemParams {
    pub id: String,
    pub effect_id: String,
    pub texture_id: String,
    pub animation: AnimatedSpriteMetadata,
}
