extern crate core;

#[macro_use]
pub mod error;
pub mod audio;
pub mod camera;
pub mod channel;
pub mod color;
pub mod config;
pub mod drawables;
pub mod ecs;
pub mod event;
pub mod file;
pub mod game;
pub mod gui;
pub mod input;
pub mod map;
pub mod math;
pub mod network;
pub mod noise;
pub mod parsing;
pub mod particles;
pub mod physics;
pub mod prelude;
pub mod rendering;
pub mod resources;
pub mod state;
pub mod storage;
pub mod text;
pub mod texture;
pub mod transform;
pub mod video;
pub mod viewport;
pub mod window;

pub use config::Config;
pub use error::{Error, Result};

pub use macros::*;

cfg_if! {
    if #[cfg(feature = "internal-backend")] {
        pub use glutin;

        #[cfg(target_arch = "wasm32")]
        pub use wasm_bindgen;

        #[cfg(not(target_arch = "wasm32"))]
        pub use tokio;

        #[path = "backend_impl/internal.rs"]
        pub(crate) mod backend_impl;

        pub use backend_impl::gl;
    } else if #[cfg(feature = "macroquad-backend")] {
        #[path = "backend_impl/macroquad.rs"]
        pub(crate) mod backend_impl;

        pub use macroquad;
    } else {
        panic!("No backend has been selected");
    }
}

cfg_if! {
    if #[cfg(feature = "platformer-physics")] {
        #[path = "physics_impl/platformer.rs"]
        pub(crate) mod physics_impl;
    } else {
        pub(crate) mod physics_impl {}
    }
}

pub use quad_rand as rand;

pub use async_trait::async_trait;
pub use cfg_if::cfg_if;
pub use serde;
pub use serde_json;

pub async fn init<'a, P: Into<Option<&'a str>>>(
    seed: u64,
    assets_dir: P,
    mods_dir: P,
) -> Result<()> {
    if let Some(assets_dir) = assets_dir.into() {
        resources::set_assets_dir(assets_dir);
    }

    if let Some(mods_dir) = mods_dir.into() {
        resources::set_mods_dir(mods_dir);
    }

    rand::srand(seed);

    input::init_gamepad_context().await?;

    Ok(())
}
