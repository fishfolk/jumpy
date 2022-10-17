use std::sync::Arc;

use bevy::{
    asset::{Asset, AssetPath, LoadState},
    reflect::FromReflect,
};
use bevy_has_load_progress::{HasLoadProgress, LoadProgress, LoadingResources};

use crate::prelude::*;

/// A wrapper around [`Handle<T>`] that also contains the [`AssetPath`] used to load the asset.
///
/// Unlike [`Handle<T>`], this type is serializable and deserializable and can be sent over the
/// network.
///
/// > **ℹ️ Note:** When deserializing an [`AssetHandle`] you will always get a **weak** handle.
#[derive(Component, Reflect, FromReflect)]
#[reflect_value(Component, Serialize, Deserialize)]
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
        let asset_path = AssetPath::deserialize(deserializer)?;
        let handle = Handle::<T>::weak(asset_path.get_id().into());

        Ok(Self {
            inner: handle,
            asset_path: Arc::new(asset_path),
        })
    }
}

// Implement `HasLoadProgress` for asset handles
impl<T: Asset> HasLoadProgress for AssetHandle<T> {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        let loaded =
            loading_resources.asset_server.get_load_state(&self.inner) == LoadState::Loaded;

        LoadProgress {
            loaded: if loaded { 1 } else { 0 },
            total: 1,
        }
    }
}
