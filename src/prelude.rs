//! Internal prelude used to easily import common types.

pub use crate::{
    assets::*, audio::*, bevy_states::*, camera::*, config::*, console::*, debug::*, input::*,
    loading::*, localization::*, logs::*, metadata::*, platform::*, session::*, ui::*, utils::*, *,
};
pub use anyhow::Context;
pub use jumpy_core::bevy_prelude::*;

pub use {
    bevy::{
        ecs::system::SystemParam,
        prelude::*,
        render::view::RenderLayers,
        // time::FixedTimestep,
        utils::{HashMap, HashSet},
    },
    bevy_egui::egui,
    bevy_kira_audio::prelude::*,
    bevy_kira_audio::AudioSource,
    bones_bevy_asset::{BonesBevyAsset, BonesBevyAssetLoad},
    bones_lib::prelude as bones,
    leafwing_input_manager::prelude::*,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    std::{marker::PhantomData, ops::Deref, sync::Arc},
    type_ulid::TypeUlid,
};
