use std::{
    path::Path,
    collections::HashMap,
};

use macroquad::{
    audio::{
        Sound,
        load_sound,
    },
    prelude::*,
};

use macroquad_particles::{Emitter, EmittersCache};

use serde::{
    Serialize,
    Deserialize,
};

use crate::{
    items::ITEMS,
    editor::DEFAULT_TOOL_ICON_TEXTURE_ID,
    json,
};

#[derive(Serialize, Deserialize)]
struct SoundResource {
    id: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct TextureResource {
    id: String,
    path: String,
    #[serde(default, with = "json::def_uvec2", skip_serializing_if = "json::uvec2_is_zero")]
    frame_size: UVec2,
    #[serde(default = "json::default_filter_mode", with = "json::FilterModeDef")]
    filter_mode: FilterMode,
}

#[derive(Copy, Clone)]
pub struct TextureEntry {
    pub texture: Texture2D,
    pub frame_size: UVec2,
}

#[derive(Serialize, Deserialize)]
struct MapResource {
    pub path: String,
    pub preview_path: String,
    #[serde(default)]
    pub is_tiled: bool,
}

#[derive(Clone)]
pub struct MapEntry {
    pub path: String,
    pub preview: Texture2D,
    pub is_tiled: bool,
}

pub struct Resources {
    // These should be moved somewhere else.
    // Optimally, Resources should hold only game data and assets, not state, like these
    // Emitters and EmitterCaches...
    pub hit_emitters: EmittersCache,
    pub explosion_emitters: EmittersCache,
    pub life_ui_explosion_emitters: EmittersCache,
    pub explosion_fire_emitter: Emitter,
    pub explosion_particle_emitters: EmittersCache,
    pub smoke_emitter: Emitter,
    pub item_emitters: HashMap<String, EmittersCache>,

    pub sounds: HashMap<String, Sound>,
    pub music: HashMap<String, Sound>,
    pub textures: HashMap<String, TextureEntry>,
    pub maps: Vec<MapEntry>,
}

impl Resources {
    const EFFECTS_DIR: &'static str = "effects";

    const SOUNDS_DIR: &'static str = "sounds";
    const SOUNDS_FILE: &'static str = "sounds.ron";

    const MUSIC_DIR: &'static str = "music";
    const MUSIC_FILE: &'static str = "music.ron";

    const TEXTURES_DIR: &'static str = "textures";
    const TEXTURES_FILE: &'static str = "textures.ron";

    const MAPS_DIR: &'static str = "maps";
    const MAPS_FILE: &'static str = "maps.ron";

    // TODO: fix macroquad error type here
    pub async fn new(assets_dir: &str) -> Result<Resources, macroquad::prelude::FileError> {
        let assets_dir = Path::new(assets_dir);

        let effects_dir = assets_dir.join(Self::EFFECTS_DIR);

        let hit_emitters = {
            let file_path = effects_dir.join("hit.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let explosion_emitters = {
            let file_path = effects_dir.join("explosion.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let life_ui_explosion_emitters = {
            let file_path = effects_dir.join("life_ui_explosion.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let explosion_fire_emitter = {
            let file_path = effects_dir.join("cannonball_hit.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            Emitter::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let explosion_particle_emitters = {
            let file_path = effects_dir.join("explosion_particles.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let smoke_emitter = {
            let file_path = effects_dir.join("smoke.json");
            let json = load_string(&file_path.to_string_lossy()).await?;
            Emitter::new(nanoserde::DeJson::deserialize_json(&json).unwrap())
        };

        let mut sounds = HashMap::new();

        {
            let sounds_dir = assets_dir.join(Self::SOUNDS_DIR);
            let sounds_file = sounds_dir.join(Self::SOUNDS_FILE);

            let bytes = load_file(&sounds_file.to_string_lossy()).await?;
            let sound_resources: Vec<SoundResource> = ron::de::from_bytes(&bytes).unwrap();

            for res in sound_resources {
                let file_path = sounds_dir.join(res.path);

                let sound = load_sound(&file_path.to_string_lossy()).await?;

                sounds.insert(res.id, sound);
            }
        }

        let mut music = HashMap::new();

        {
            let music_dir = assets_dir.join(Self::MUSIC_DIR);
            let music_file = music_dir.join(Self::MUSIC_FILE);

            let bytes = load_file(&music_file.to_string_lossy()).await?;
            let music_resources: Vec<SoundResource> = ron::de::from_bytes(&bytes).unwrap();

            for res in music_resources {
                let file_path = music_dir.join(res.path);

                let sound = load_sound(&file_path.to_string_lossy()).await?;

                music.insert(res.id, sound);
            }
        }

        let mut textures = HashMap::new();

        {
            let textures_dir = assets_dir.join(Self::TEXTURES_DIR);
            let textures_file = textures_dir.join(Self::TEXTURES_FILE);

            let bytes = load_file(&textures_file.to_string_lossy()).await?;
            let texture_resources: Vec<TextureResource> = ron::de::from_bytes(&bytes).unwrap();

            for res in texture_resources {
                let file_path = textures_dir.join(res.path);

                let texture = load_texture(&file_path.to_string_lossy()).await?;
                texture.set_filter(res.filter_mode);

                let mut frame_size = res.frame_size;
                if frame_size == UVec2::ZERO {
                    frame_size = uvec2(texture.width() as u32, texture.height() as u32)
                }

                let entry = TextureEntry {
                    texture,
                    frame_size,
                };

                textures.insert(res.id, entry);
            }
        }

        let mut maps = Vec::new();

        {
            let maps_dir = assets_dir.join(Self::MAPS_DIR);
            let maps_file = maps_dir.join(Self::MAPS_FILE);

            let bytes = load_file(&maps_file.to_string_lossy()).await?;
            let map_resources: Vec<MapResource> = ron::de::from_bytes(&bytes).unwrap();

            for res in map_resources {
                let map_path = maps_dir.join(res.path);
                let preview_path = maps_dir.join(res.preview_path);

                let preview = load_texture(&preview_path.to_string_lossy()).await?;

                let entry = MapEntry {
                    path: map_path.to_string_lossy().into(),
                    preview,
                    is_tiled: res.is_tiled,
                };

                maps.push(entry)
            }
        }

        #[allow(clippy::inconsistent_struct_constructor)]
            Ok(Resources {
            hit_emitters,
            explosion_emitters,
            life_ui_explosion_emitters,
            smoke_emitter,
            explosion_fire_emitter,
            explosion_particle_emitters,
            item_emitters: HashMap::new(),

            sounds,
            music,
            textures,
            maps,
        })
    }
}
