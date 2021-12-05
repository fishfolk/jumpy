use std::{collections::HashMap, fs, path::Path};

use macroquad::{
    audio::{load_sound, Sound},
    experimental::{collections::storage, coroutines::start_coroutine},
    prelude::*,
};

use ff_particles::EmitterConfig;

use serde::{Deserialize, Serialize};

use crate::gui::GuiResources;
use crate::{
    error::{ErrorKind, Result},
    formaterr,
    items::ItemParams,
    json::{self, deserialize_bytes},
    map::Map,
};

use crate::player::PlayerCharacterParams;
use crate::text::ToStringHelper;

#[derive(Serialize, Deserialize)]
struct ParticleEffectMetadata {
    id: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct SoundMetadata {
    id: String,
    path: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureKind {
    Background,
    Tileset,
    Spritesheet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<TextureKind>,
    #[serde(
        default,
        with = "json::uvec2_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub sprite_size: Option<UVec2>,
    #[serde(default = "json::default_filter_mode", with = "json::FilterModeDef")]
    pub filter_mode: FilterMode,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct TextureResource {
    pub texture: Texture2D,
    pub meta: TextureMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: String,
    pub path: String,
    #[serde(default, skip)]
    pub size: Vec2,
}

#[derive(Debug, Clone)]
pub struct ImageResource {
    pub image: Image,
    pub meta: ImageMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub path: String,
    pub preview_path: String,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub is_tiled_map: bool,
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub is_user_map: bool,
}

#[derive(Debug, Clone)]
pub struct MapResource {
    pub map: Map,
    pub preview: Texture2D,
    pub meta: MapMetadata,
}

pub struct Resources {
    pub assets_dir: String,

    pub particle_effects: HashMap<String, EmitterConfig>,
    pub sounds: HashMap<String, Sound>,
    pub music: HashMap<String, Sound>,
    pub textures: HashMap<String, TextureResource>,
    pub images: HashMap<String, ImageResource>,
    pub maps: Vec<MapResource>,
    pub items: HashMap<String, ItemParams>,
    pub player_characters: Vec<PlayerCharacterParams>,
}

impl Resources {
    pub const PARTICLE_EFFECTS_DIR: &'static str = "particle_effects";
    pub const SOUNDS_FILE: &'static str = "sounds";
    pub const MUSIC_FILE: &'static str = "music";
    pub const TEXTURES_FILE: &'static str = "textures";
    pub const IMAGES_FILE: &'static str = "images";
    pub const MAPS_FILE: &'static str = "maps";
    pub const ITEMS_FILE: &'static str = "items";
    pub const PLAYER_CHARACTERS_FILE: &'static str = "player_characters";

    pub const RESOURCE_FILES_EXTENSION: &'static str = "json";

    pub const MAP_EXPORTS_EXTENSION: &'static str = "json";
    pub const MAP_EXPORTS_DEFAULT_DIR: &'static str = "maps";
    pub const MAP_EXPORT_NAME_MIN_LEN: usize = 1;

    pub const MAP_PREVIEW_PLACEHOLDER_PATH: &'static str = "maps/no_preview.png";
    pub const MAP_PREVIEW_PLACEHOLDER_ID: &'static str = "map_preview_placeholder";

    pub async fn new(assets_dir: &str) -> Result<Resources> {
        let assets_dir_path = Path::new(assets_dir);

        let mut particle_effects = HashMap::new();

        {
            let particle_effects_file_path = assets_dir_path
                .join(Self::PARTICLE_EFFECTS_DIR)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&particle_effects_file_path.to_string_helper()).await?;
            let metadata: Vec<ParticleEffectMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let file_path = assets_dir_path.join(&meta.path);

                let bytes = load_file(&file_path.to_string_helper()).await?;
                let cfg: EmitterConfig = serde_json::from_slice(&bytes)?;

                particle_effects.insert(meta.id, cfg);
            }
        }

        let mut sounds = HashMap::new();

        {
            let sounds_file_path = assets_dir_path
                .join(Self::SOUNDS_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&sounds_file_path.to_string_helper()).await?;
            let metadata: Vec<SoundMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let file_path = assets_dir_path.join(meta.path);

                let sound = load_sound(&file_path.to_string_helper()).await?;

                sounds.insert(meta.id, sound);
            }
        }

        let mut music = HashMap::new();

        {
            let music_file_path = assets_dir_path
                .join(Self::MUSIC_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&music_file_path.to_string_helper()).await?;
            let metadata: Vec<SoundMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let file_path = assets_dir_path.join(meta.path);

                let sound = load_sound(&file_path.to_string_helper()).await?;

                music.insert(meta.id, sound);
            }
        }

        let mut textures = HashMap::new();

        {
            let textures_file_path = assets_dir_path
                .join(Self::TEXTURES_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&textures_file_path.to_string_helper()).await?;
            let metadata: Vec<TextureMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let file_path = assets_dir_path.join(&meta.path);

                let texture = load_texture(&file_path.to_string_helper()).await?;
                texture.set_filter(meta.filter_mode);

                let sprite_size = {
                    let val = meta
                        .sprite_size
                        .unwrap_or_else(|| vec2(texture.width(), texture.height()).as_u32());

                    Some(val)
                };

                let size = vec2(texture.width(), texture.height());

                let key = meta.id.clone();

                let meta = TextureMetadata {
                    sprite_size,
                    size,
                    ..meta
                };

                let res = TextureResource { texture, meta };

                textures.insert(key, res);
            }
        }

        let mut images = HashMap::new();

        {
            let images_file_path = assets_dir_path
                .join(Self::IMAGES_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&images_file_path.to_string_helper()).await?;
            let metadata: Vec<ImageMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let file_path = assets_dir_path.join(&meta.path);

                let image = load_image(&file_path.to_string_helper()).await?;

                let key = meta.id.clone();

                let meta = ImageMetadata {
                    size: vec2(image.width() as f32, image.height() as f32),
                    ..meta
                };

                let res = ImageResource { image, meta };

                images.insert(key, res);
            }
        }

        let mut maps = Vec::new();

        {
            let maps_file_path = assets_dir_path
                .join(Self::MAPS_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&maps_file_path.to_string_helper()).await?;
            let metadata: Vec<MapMetadata> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for meta in metadata {
                let map_path = assets_dir_path.join(&meta.path);
                let preview_path = assets_dir_path.join(&meta.preview_path);

                let map = if meta.is_tiled_map {
                    Map::load_tiled(map_path, None).await?
                } else {
                    Map::load(map_path).await?
                };

                let preview = load_texture(&preview_path.to_string_helper()).await?;

                let res = MapResource { map, preview, meta };

                maps.push(res)
            }
        }

        let mut items = HashMap::new();

        {
            let items_file_path = assets_dir_path
                .join(Self::ITEMS_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&items_file_path.to_string_helper()).await?;
            let item_paths: Vec<String> =
                deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

            for path in item_paths {
                let path = assets_dir_path.join(&path);

                let bytes = load_file(&path.to_string_helper()).await?;

                let params: ItemParams = deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?;

                items.insert(params.id.clone(), params);
            }
        }

        let player_characters = {
            let player_characters_file_path = assets_dir_path
                .join(Self::PLAYER_CHARACTERS_FILE)
                .with_extension(Self::RESOURCE_FILES_EXTENSION);

            let bytes = load_file(&player_characters_file_path.to_string_helper()).await?;
            deserialize_bytes(Self::RESOURCE_FILES_EXTENSION, &bytes)?
        };

        #[allow(clippy::inconsistent_struct_constructor)]
        Ok(Resources {
            assets_dir: assets_dir.to_string(),
            particle_effects,
            sounds,
            music,
            textures,
            images,
            maps,
            items,
            player_characters,
        })
    }

    pub fn create_map(
        &self,
        name: &str,
        description: Option<&str>,
        tile_size: Vec2,
        grid_size: UVec2,
    ) -> Result<MapResource> {
        let description = description.map(|str| str.to_string());

        let map_path = Path::new(Self::MAP_EXPORTS_DEFAULT_DIR)
            .join(map_name_to_filename(name))
            .with_extension(Self::MAP_EXPORTS_EXTENSION);

        let path = map_path.to_string_helper();

        let preview_path = Path::new(Self::MAP_PREVIEW_PLACEHOLDER_PATH).to_string_helper();

        let meta = MapMetadata {
            name: name.to_string(),
            description,
            path,
            preview_path,
            is_tiled_map: false,
            is_user_map: true,
        };

        let map = Map::new(tile_size, grid_size);

        let preview = {
            let res = self.textures.get(Self::MAP_PREVIEW_PLACEHOLDER_ID).unwrap();
            res.texture
        };

        Ok(MapResource { map, preview, meta })
    }

    pub fn save_map(&mut self, map_resource: &MapResource) -> Result<()> {
        let assets_path = Path::new(&self.assets_dir);
        let export_path = assets_path.join(&map_resource.meta.path);

        if export_path.exists() {
            let mut i = 0;
            while i < self.maps.len() {
                let res = &self.maps[i];
                if res.meta.path == map_resource.meta.path {
                    if res.meta.is_user_map {
                        self.maps.remove(i);
                        break;
                    } else {
                        return Err(formaterr!(
                            ErrorKind::General,
                            "Resources: The path '{}' is in use and it is not possible to overwrite. Please choose a different map name",
                            &map_resource.meta.path,
                        ));
                    }
                }

                i += 1;
            }
        }

        map_resource.map.save(export_path)?;

        self.maps.push(map_resource.clone());
        self.save_maps_file()?;

        Ok(())
    }

    pub fn delete_map(&mut self, index: usize) -> Result<()> {
        let map_resource = self.maps.remove(index);

        let path = Path::new(&self.assets_dir).join(&map_resource.meta.path);

        fs::remove_file(path)?;

        self.save_maps_file()?;

        Ok(())
    }

    fn save_maps_file(&self) -> Result<()> {
        let maps_file_path = Path::new(&self.assets_dir)
            .join(Self::MAPS_FILE)
            .with_extension(Self::RESOURCE_FILES_EXTENSION);

        let metadata: Vec<MapMetadata> = self.maps.iter().map(|res| res.meta.clone()).collect();

        let str = serde_json::to_string_pretty(&metadata)?;
        fs::write(maps_file_path, &str)?;

        Ok(())
    }
}

pub fn map_name_to_filename(name: &str) -> String {
    name.replace(' ', "_").replace('.', "_").to_lowercase()
}

pub fn is_valid_map_file_name(file_name: &str) -> bool {
    if file_name.len() - Resources::MAP_EXPORTS_EXTENSION.len() > Resources::MAP_EXPORT_NAME_MIN_LEN
    {
        if let Some(extension) = Path::new(file_name).extension() {
            return extension == Resources::MAP_EXPORTS_EXTENSION;
        }
    }

    false
}

pub fn is_valid_map_export_path<P: AsRef<Path>>(path: P, should_overwrite: bool) -> bool {
    let path = path.as_ref();

    if let Some(file_name) = path.file_name() {
        if is_valid_map_file_name(&file_name.to_string_helper()) {
            let resources = storage::get::<Resources>();

            let res = resources
                .maps
                .iter()
                .find(|res| Path::new(&res.meta.path) == path);

            if let Some(res) = res {
                return res.meta.is_user_map && should_overwrite;
            }

            return true;
        }
    }

    false
}

pub async fn load_resources(assets_dir: &str) {
    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.to_string();
        async move {
            let resources = match Resources::new(&assets_dir).await {
                Ok(val) => val,
                Err(err) => panic!("{}: {}", err.kind().as_str(), err),
            };

            storage::store(resources);
        }
    });

    while !resources_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );

        next_frame().await;
    }

    {
        let gui_resources = GuiResources::load(assets_dir).await;
        storage::store(gui_resources);
    }
}
