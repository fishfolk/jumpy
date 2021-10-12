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
use nanoserde::Toml;

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
    pub hit_fxses: EmittersCache,
    pub explosion_fxses: EmittersCache,
    pub life_ui_explosion_fxses: EmittersCache,
    pub fx_explosion_fire: Emitter,
    pub fx_explosion_particles: EmittersCache,
    pub fx_smoke: Emitter,
    pub jump_sound: Sound,
    pub shoot_sound: Sound,
    pub sword_sound: Sound,
    pub pickup_sound: Sound,
    pub player_landing_sound: Sound,
    pub player_throw_sound: Sound,
    pub player_die_sound: Sound,
    pub items_fxses: HashMap<String, EmittersCache>,

    pub textures: HashMap<String, TextureEntry>,
    pub maps: Vec<MapEntry>,
}

impl Resources {
    const TEXTURES_DIR: &'static str = "textures";
    const TEXTURES_FILE: &'static str = "textures.ron";

    const MAPS_DIR: &'static str = "maps";
    const MAPS_FILE: &'static str = "maps.ron";

    // TODO: fix macroquad error type here
    pub async fn new(assets_dir: &str) -> Result<Resources, macroquad::prelude::FileError> {
        let jump_sound = load_sound(&format!("{}/sounds/jump.wav", assets_dir)).await?;
        let shoot_sound = load_sound(&format!("{}/sounds/shoot.ogg", assets_dir)).await?;
        let sword_sound = load_sound(&format!("{}/sounds/sword.wav", assets_dir)).await?;
        let pickup_sound = load_sound(&format!("{}/sounds/pickup.wav", assets_dir)).await?;

        let player_landing_sound =
            load_sound(&format!("{}/sounds/player_landing.wav", assets_dir)).await?;
        let player_throw_sound =
            load_sound(&format!("{}/sounds/throw_noiz.wav", assets_dir)).await?;
        let player_die_sound =
            load_sound(&format!("{}/sounds/fish_fillet.wav", assets_dir)).await?;

        const HIT_FX: &str = include_str!("../assets/fxses/hit.json");
        const EXPLOSION_FX: &str = include_str!("../assets/fxses/explosion.json");
        const LIFE_UI_FX: &str = include_str!("../assets/fxses/life_ui_explosion.json");
        const CANNONBALL_HIT_FX: &str = include_str!("../assets/fxses/canonball_hit.json");
        const EXPLOSION_PARTICLES: &str = include_str!("../assets/fxses/explosion_particles.json");
        const SMOKE_FX: &str = include_str!("../assets/fxses/smoke.json");

        let hit_fxses = EmittersCache::new(nanoserde::DeJson::deserialize_json(HIT_FX).unwrap());
        let explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_FX).unwrap());
        let life_ui_explosion_fxses =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(LIFE_UI_FX).unwrap());
        let fx_explosion_fire =
            Emitter::new(nanoserde::DeJson::deserialize_json(CANNONBALL_HIT_FX).unwrap());
        let fx_explosion_particles =
            EmittersCache::new(nanoserde::DeJson::deserialize_json(EXPLOSION_PARTICLES).unwrap());
        let fx_smoke = Emitter::new(nanoserde::DeJson::deserialize_json(SMOKE_FX).unwrap());

        let mut items_fxses = HashMap::new();
        for item in ITEMS {
            for (id, path) in item.fxses {
                let json = load_string(path).await?;
                let emitter_cache =
                    EmittersCache::new(nanoserde::DeJson::deserialize_json(&json).unwrap());
                items_fxses.insert(
                    format!("{}/{}", item.tiled_name, id.to_string()),
                    emitter_cache,
                );
            }
        }

        let assets_dir = Path::new(assets_dir);

        let mut textures = HashMap::new();

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

        let mut maps = Vec::new();

        let maps_dir = assets_dir.join(Self::MAPS_DIR);
        let maps_file = maps_dir.join(Self::MAPS_FILE);

        let bytes= load_file(&maps_file.to_string_lossy()).await?;
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

        #[allow(clippy::inconsistent_struct_constructor)]
            Ok(Resources {
            hit_fxses,
            explosion_fxses,
            life_ui_explosion_fxses,
            fx_smoke,
            items_fxses,
            jump_sound,
            shoot_sound,
            sword_sound,
            pickup_sound,
            player_landing_sound,
            player_throw_sound,
            player_die_sound,
            fx_explosion_fire,
            fx_explosion_particles,
            textures,
            maps,
        })
    }
}
