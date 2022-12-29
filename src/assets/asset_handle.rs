use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::asset::{Asset, AssetPath};
use bones_bevy_asset::BonesBevyAssetLoad;
use normalize_path::NormalizePath;

use crate::prelude::*;

/// A wrapper around [`Handle<T>`] that also contains the [`AssetPath`] used to load the asset.
///
/// Unlike [`Handle<T>`], this type is serializable and deserializable and can be sent over the
/// network.
///
/// > **ℹ️ Note:** When deserializing an [`AssetHandle`] you will always get a **weak** handle.
#[derive(Component)]
pub struct AssetHandle<T: Asset> {
    pub inner: Handle<T>,
    pub asset_path: Arc<AssetPath<'static>>,
}

impl<T: Asset> Default for AssetHandle<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            asset_path: Arc::new("<dummy_path>".into()),
        }
    }
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            asset_path: self.asset_path.clone(),
        }
    }
}

impl<T: Asset> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("handle", &self.inner)
            .field("asset_path", &self.asset_path)
            .finish()
    }
}

impl<T: Asset> std::ops::Deref for AssetHandle<T> {
    type Target = Handle<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(asset_path: AssetPath<'static>, handle: Handle<T>) -> Self {
        Self {
            asset_path: Arc::new(asset_path),
            inner: handle,
        }
    }

    pub fn clone_weak(&self) -> Self {
        Self {
            inner: self.inner.clone_weak(),
            asset_path: self.asset_path.clone(),
        }
    }
}

impl<T: Asset> Serialize for AssetHandle<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.asset_path.serialize(serializer)
    }
}

impl<'de, T: Asset> Deserialize<'de> for AssetHandle<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let asset_path = String::deserialize(deserializer)?;
        let asset_path: AssetPath = asset_path.into();
        let handle = Handle::<T>::weak(asset_path.get_id().into());

        Ok(Self {
            inner: handle,
            asset_path: Arc::new(asset_path),
        })
    }
}

impl<T: Asset> BonesBevyAssetLoad for AssetHandle<T> {
    fn load(
        &mut self,
        load_context: &mut bevy::asset::LoadContext,
        dependencies: &mut Vec<bevy::asset::AssetPath<'static>>,
    ) {
        let base_path = load_context.path();
        let new_path = get_normalized_relative_path(base_path, self.asset_path.path());
        let new_path = AssetPath::new(new_path, self.asset_path.label().map(|x| x.to_owned()));
        dependencies.push(new_path.clone());
        self.asset_path = Arc::new(new_path);

        self.inner = load_context.get_handle(self.asset_path.get_id());
    }
}

/// Calculate an asset's full path relative to another asset
fn get_normalized_relative_path(base_path: &Path, relative_path: &Path) -> PathBuf {
    let is_relative = !relative_path.starts_with("/");

    let path = if is_relative {
        let base = base_path.parent().unwrap_or_else(|| Path::new(""));
        base.join(relative_path)
    } else {
        Path::new(relative_path)
            .strip_prefix("/")
            .unwrap()
            .to_owned()
    };
    path.normalize()
}
