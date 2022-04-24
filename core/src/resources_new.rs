use core::panicking::panic;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;

pub trait Asset {}

struct ResourceAssets {
    asset_ids: HashMap<String, usize>,
    assets: HashMap<usize, Box<dyn Any>>,
}

impl ResourceAssets {
    fn try_get_asset<T: Any + Asset>(&self, id: &str) -> Option<impl Deref<Target = T>> {
        if let Some(index) = self.asset_ids.get(id) {
            self.try_get_asset_index(*index)
        } else {
            None
        }
    }

    fn get_asset<T: Any + Asset>(&self, id: &str) -> impl Deref<Target = T> {
        self.try_get_asset(id).unwrap()
    }

    fn try_get_asset_index<T: Any + Asset>(&self, index: usize) -> Option<impl Deref<Target = T>> {
        self.assets.get(&index)
    }

    fn get_asset_index<T: Any + Asset>(&self, index: usize) -> impl Deref<Target = T> {
        self.try_get_asset_index(index).unwrap()
    }
}

pub struct Resources {
    loaded_mods: Vec<ModMetadata>,
    resources: HashMap<TypeId, ResourceAssets>,
}

impl Resources {
    fn get_resource<T: Any + Asset>(&self) -> &ResourceAssets {
        self.resources.get(&TypeId::of::<T>()).unwrap()
    }
}

static mut RESOURCES: Option<Resources> = None;

pub fn resources() -> &'static Resources {
    unsafe {
        RESOURCES.as_ref().unwrap_or_else(|| {
            panic!("ERROR: Attempting to access resources before they have been loaded!"))
        })
    }
}

pub fn resources_mut() -> &'static mut Resources {
    unsafe {
        RESOURCES.as_mut().unwrap_or_else(|| {
            panic!("ERROR: Attempting to access resources before they have been loaded!"))
        })
    }
}

const DEFAULT_RESOURCE_FILE_EXTENSION: &str = "json";

pub struct ResourceLoader {
    assets_dir: String,
    mods_dir: Option<String>,
    file_extension: String,
}

impl ResourceLoader {
    pub fn new() -> Self {
        ResourceLoader {
            assets_dir: DEFAULT_ASSETS_PATH.to_string(),
            mods_dir: None,
            file_extension: DEFAULT_RESOURCE_FILE_EXTENSION.to_string(),
        }
    }

    pub fn with_assets_dir<P: AsRef<Path>>(self, path: P) -> Self {
        let path = path.as_ref();

        ResourceLoader {
            assets_dir: path.to_string_lossy().to_string(),
            ..self
        }
    }

    pub fn with_mods_dir<P: AsRef<Path>>(self, path: P) -> Self {
        let path = path.as_ref();

        ResourceLoader {
            mods_dir: Some(path.to_string_lossy().to_string()),
            ..self
        }
    }

    pub fn with_file_extension(self, extension: &str) -> Self {
        let extension = if extension.starts_with('.') {
            extension.slice(1..)
        } else {
            extension
        };

        ResourceLoader {
            file_extension: extension.to_string(),
            ..self
        }
    }

    pub fn with_resource<T: Resource>(self) -> Self {

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