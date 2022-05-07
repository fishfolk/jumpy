use hecs::{Entity, World};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::drawables::{AnimatedSpriteMetadata, Drawable};
use crate::file::read_from_file;
use crate::math::Vec2;
use crate::parsing::deserialize_bytes_by_extension;
use crate::result::Result;
use crate::texture::get_texture;
use crate::transform::Transform;

const DECORATION_DRAW_ORDER: u32 = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct DecorationMetadata {
    pub id: String,
    pub sprite: AnimatedSpriteMetadata,
}

pub struct Decoration {
    pub id: String,
}

impl Decoration {
    pub fn new(id: &str) -> Self {
        Decoration { id: id.to_string() }
    }
}

pub fn spawn_decoration(world: &mut World, position: Vec2, meta: DecorationMetadata) -> Entity {
    let texture = get_texture(&meta.sprite.texture_id);

    let animations = meta
        .sprite
        .animations
        .clone()
        .into_iter()
        .map(|m| m.into())
        .collect::<Vec<_>>();

    world.spawn((
        Decoration::new(&meta.id),
        Transform::from(position),
        Drawable::new_animated_sprite(
            DECORATION_DRAW_ORDER,
            texture,
            texture.frame_size(),
            animations.as_slice(),
            meta.sprite.clone().into(),
        ),
    ))
}

const DECORATION_RESOURCES_FILE: &str = "decoration";

static mut DECORATION: Option<HashMap<String, DecorationMetadata>> = None;

pub fn try_get_decoration(id: &str) -> Option<&DecorationMetadata> {
    unsafe { DECORATION.get_or_insert_with(HashMap::new).get(id) }
}

pub fn get_decoration(id: &str) -> &DecorationMetadata {
    try_get_decoration(id).unwrap()
}

pub fn iter_decoration() -> Iter<'static, String, DecorationMetadata> {
    unsafe { DECORATION.get_or_insert_with(HashMap::new) }.iter()
}

pub async fn load_decoration<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let decoration = unsafe { DECORATION.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        decoration.clear();
    }

    let decoration_file_path = path
        .as_ref()
        .join(DECORATION_RESOURCES_FILE)
        .with_extension(ext);

    match read_from_file(&decoration_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let decoration_paths: Vec<String> = deserialize_bytes_by_extension(ext, &bytes)?;

            for decoration_path in decoration_paths {
                let path = path.as_ref().join(&decoration_path);

                let extension = path.extension().unwrap().to_str().unwrap();

                let bytes = read_from_file(&path).await?;

                let params: DecorationMetadata = deserialize_bytes_by_extension(extension, &bytes)?;

                decoration.insert(params.id.clone(), params);
            }
        }
    }

    Ok(())
}
