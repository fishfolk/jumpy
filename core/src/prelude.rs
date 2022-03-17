pub use cfg_if::cfg_if;

pub use crate::file::load_file;

pub use crate::audio::*;
pub use crate::math::*;
pub use crate::config::*;
pub use crate::parsing::*;
pub use crate::channel::*;
pub use crate::transform::*;
pub use crate::events::*;
pub use crate::ecs::*;
pub use crate::input::*;
pub use crate::rendering::*;
pub use crate::viewport::*;
pub use crate::window::*;
pub use crate::texture::*;
pub use crate::color::*;
pub use crate::game::*;
pub use crate::drawables::*;
pub use crate::particles::*;

pub use crate::resources::{
    assets_dir, mods_dir, loaded_mods,
    try_get_texture, get_texture, iter_textures, try_get_decoration, get_decoration, iter_decoration, try_get_font, get_font,
    try_get_sound, get_sound, try_get_map, get_map, iter_maps, try_get_particle_effect, get_particle_effect, iter_particle_effects,
};

#[cfg(feature = "macroquad")]
pub use crate::resources::{try_get_image, get_image, iter_images};

pub use macros::*;

pub use crate::rand;
pub use crate::storage;