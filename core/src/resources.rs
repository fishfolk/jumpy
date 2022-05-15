use std::slice::{Iter, IterMut};
use std::{collections::HashMap, path::Path};

use async_trait::async_trait;

use serde::{Deserialize, Serialize};

use serde::de::DeserializeOwned;

use crate::prelude::*;
use crate::result::Result;

pub const DEFAULT_RESOURCE_FILE_EXTENSION: &str = "json";

const ACTIVE_MODS_FILE_NAME: &str = "active_mods";
const MOD_FILE_NAME: &str = "fishfight_mod";

pub trait Resource: Clone + DeserializeOwned {}

pub trait ResourceId: Resource {
    fn id(&self) -> String;
}

#[async_trait]
pub trait ResourceMap: ResourceId {
    fn storage() -> &'static HashMap<String, Self>;

    async fn load<P: AsRef<Path> + Send, E: Into<Option<&'static str>> + Send>(
        path: P,
        ext: E,
        is_required: bool,
        should_overwrite: bool,
    ) -> Result<()>;

    fn iter() -> std::collections::hash_map::Iter<'static, String, Self> {
        Self::storage().iter()
    }

    fn try_get(id: &str) -> Option<&'static Self> {
        Self::storage().get(id)
    }

    fn get(id: &str) -> &'static Self {
        Self::try_get(id).unwrap()
    }
}

pub trait ResourceMapMut: ResourceMap {
    fn storage_mut() -> &'static mut HashMap<String, Self>;

    fn iter_mut() -> std::collections::hash_map::IterMut<'static, String, Self> {
        Self::storage_mut().iter_mut()
    }

    fn try_get_mut(id: &str) -> Option<&'static mut Self> {
        Self::storage_mut().get_mut(id)
    }

    fn get_mut(id: &str) -> &'static mut Self {
        Self::try_get_mut(id).unwrap()
    }
}

#[async_trait]
pub trait ResourceVec: Resource {
    fn storage() -> &'static Vec<Self>;

    async fn load<P: AsRef<Path> + Send, E: Into<Option<&'static str>> + Send>(
        path: P,
        ext: E,
        is_required: bool,
        should_overwrite: bool,
    ) -> Result<()>;

    fn iter() -> Iter<'static, Self> {
        Self::storage().iter()
    }

    fn try_get(index: usize) -> Option<&'static Self> {
        Self::storage().get(index)
    }

    fn get(index: usize) -> &'static Self {
        Self::try_get(index).unwrap()
    }
}

pub trait ResourceVecMut: ResourceVec {
    fn storage_mut() -> &'static mut Vec<Self>;

    fn iter_mut() -> IterMut<'static, Self> {
        Self::storage_mut().iter_mut()
    }

    fn try_get_mut(index: usize) -> Option<&'static mut Self> {
        Self::storage_mut().get_mut(index)
    }

    fn get_mut(index: usize) -> &'static mut Self {
        Self::try_get_mut(index).unwrap()
    }
}

const DEFAULT_ASSETS_DIR: &str = "assets/";

static mut ASSETS_DIR: Option<String> = None;

pub fn set_assets_dir<P: AsRef<Path>>(path: P) {
    let str = path.as_ref().to_string_lossy().to_string();
    unsafe {
        ASSETS_DIR = Some(str);
    }
}

pub fn assets_dir() -> String {
    unsafe {
        ASSETS_DIR
            .get_or_insert_with(|| DEFAULT_ASSETS_DIR.to_string())
            .clone()
    }
}

const DEFAULT_MODS_DIR: &str = "mods/";

static mut MODS_DIR: Option<String> = None;

pub fn set_mods_dir<P: AsRef<Path>>(path: P) {
    let str = path.as_ref().to_string_lossy().to_string();
    unsafe {
        MODS_DIR = Some(str);
    }
}

pub fn mods_dir() -> String {
    unsafe {
        MODS_DIR
            .get_or_insert_with(|| DEFAULT_MODS_DIR.to_string())
            .clone()
    }
}

static mut LOADED_MODS: Vec<ModMetadata> = Vec::new();

pub fn loaded_mods() -> &'static [ModMetadata] {
    unsafe { LOADED_MODS.as_slice() }
}

pub(crate) fn loaded_mods_mut() -> &'static mut Vec<ModMetadata> {
    unsafe { LOADED_MODS.as_mut() }
}

pub struct ModLoadingIterator {
    extension: &'static str,
    active_mods: Vec<String>,
    next_i: usize,
}

impl ModLoadingIterator {
    pub async fn new<P: AsRef<Path>, E: Into<Option<&'static str>>>(
        path: P,
        ext: E,
    ) -> Result<Self> {
        let path = path.as_ref();

        set_mods_dir(path);

        let ext = ext.into().unwrap_or(DEFAULT_RESOURCE_FILE_EXTENSION);

        let active_mods_file_path = path.join(ACTIVE_MODS_FILE_NAME).with_extension(ext);

        let bytes = read_from_file(active_mods_file_path).await?;

        let active_mods: Vec<String> = deserialize_bytes_by_extension(ext, &bytes)?;

        Ok(ModLoadingIterator {
            extension: ext,
            active_mods,
            next_i: 0,
        })
    }

    pub async fn next(&mut self) -> Result<Option<(String, ModMetadata)>> {
        if self.next_i >= self.active_mods.len() {
            return Ok(None);
        }

        let current_mod = &self.active_mods[self.next_i];

        let mod_path = Path::new(&mods_dir()).join(current_mod);

        let mod_file_path = mod_path.join(MOD_FILE_NAME).with_extension(self.extension);

        let bytes = read_from_file(mod_file_path).await?;

        let meta: ModMetadata = deserialize_bytes_by_extension(self.extension, &bytes)?;

        let mut has_game_version_mismatch = false;

        if let Some(req_version) = &meta.game_version {
            if *req_version != env!("CARGO_PKG_VERSION") {
                has_game_version_mismatch = true;

                #[cfg(debug_assertions)]
                println!(
                    "WARNING: Loading mod {} (v{}) failed: Game version requirement mismatch (v{})",
                    &meta.id, &meta.version, req_version
                );
            }
        }

        if !has_game_version_mismatch {
            let mut has_unmet_dependencies = false;

            for dependency in &meta.dependencies {
                let res = loaded_mods()
                    .iter()
                    .find(|&meta| meta.id == dependency.id && meta.version == dependency.version);

                if res.is_none() {
                    has_unmet_dependencies = true;

                    #[cfg(debug_assertions)]
                    println!(
                        "WARNING: Loading mod {} (v{}) failed: Unmet dependency {} (v{})",
                        &meta.id, &meta.version, &dependency.id, &dependency.version
                    );

                    break;
                }
            }

            if !has_unmet_dependencies {
                loaded_mods_mut().push(meta.clone());

                self.next_i += 1;

                return Ok(Some((mod_path.to_string_lossy().to_string(), meta)));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub kind: ModKind,
    #[serde(default)]
    pub dependencies: Vec<DependencyMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetadata {
    pub id: String,
    pub version: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModKind {
    DataOnly,
    Full,
}

impl Default for ModKind {
    fn default() -> Self {
        ModKind::Full
    }
}
