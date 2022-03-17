#[macro_use]
pub mod error;
pub mod config;
pub mod parsing;
pub mod json;
pub mod noise;
pub mod text;
pub mod network;
pub mod channel;
pub mod transform;
pub mod events;
pub mod video;
pub mod file;
pub mod color;
pub mod audio;
pub mod prelude;
pub mod texture;
pub mod ecs;
pub mod rendering;
pub mod viewport;
pub mod storage;
pub mod game;
pub mod input;
pub mod window;
pub mod resources;
pub mod particles;
pub mod map;
pub mod gui;
pub mod drawables;
pub mod math;

pub use error::{Error, Result};
pub use config::Config;

pub use macros::*;

cfg_if! {
    if #[cfg(feature = "internal-backend")] {
        #[path = "backend_impl/internal.rs"]
        pub(crate) mod backend_impl;

        #[cfg(feature = "winit")]
        pub use winit;

        #[cfg(target_arch = "wasm32")]
        pub use wasm_bindgen;

        #[cfg(not(target_arch = "wasm32"))]
        pub use tokio;
    } else if #[cfg(feature = "macroquad-backend")] {
        #[path = "backend_impl/macroquad.rs"]
        pub(crate) mod backend_impl;

        pub use macroquad;
        pub use macroquad::experimental::scene;
        pub use macroquad::camera;
    }
}

pub use quad_rand as rand;

pub use async_trait::async_trait;
use cfg_if::cfg_if;