pub use cfg_if::cfg_if;

pub use crate::file::read_from_file;
pub use crate::init as init_core;

pub use crate::audio::*;
pub use crate::camera::*;
pub use crate::channel::*;
pub use crate::color::*;
pub use crate::config::*;
pub use crate::context::*;
pub use crate::drawables::*;
pub use crate::event::*;
pub use crate::game::*;
pub use crate::input::*;
pub use crate::math::*;
pub use crate::parsing::*;
pub use crate::particles::*;
pub use crate::physics::*;
pub use crate::rendering::*;
pub use crate::state::*;
pub use crate::texture::*;
pub use crate::transform::*;
pub use crate::viewport::*;
pub use crate::window::*;
pub use crate::Result;

pub use crate::ecs::{DrawFn, Entity, FixedUpdateFn, UpdateFn, World};

pub use crate::resources::{
    assets_dir, get_decoration, get_font, get_map, get_particle_effect, get_sound, get_texture,
    iter_decoration, iter_maps, iter_particle_effects, iter_textures, loaded_mods, mods_dir,
    try_get_decoration, try_get_font, try_get_map, try_get_particle_effect, try_get_sound,
    try_get_texture,
};

#[cfg(feature = "macroquad")]
pub use crate::resources::{get_image, iter_images, try_get_image};

pub use macros::*;

pub use crate::rand;
pub use crate::storage;
