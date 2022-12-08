#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::bool_to_int_with_if)]

use std::marker::PhantomData;

use bevy::{
    asset::{Asset, LoadState},
    ecs::system::SystemParam,
    math::{UVec2, Vec2, Vec3},
    prelude::{AssetServer, Entity, Handle, HandleUntyped, Res},
    utils::HashMap,
};

// Export the derive macro
pub use macros::HasLoadProgress;

/// A progress indicator holding how many items must be loaded and how many items have been loaded
#[derive(Clone, Copy, Default, Debug)]
pub struct LoadProgress {
    pub loaded: u32,
    pub total: u32,
}

impl std::fmt::Display for LoadProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {}", self.loaded, self.total)
    }
}

impl LoadProgress {
    /// Get the load progress as a percentage
    pub fn as_percent(&self) -> f32 {
        self.loaded as f32 / self.total as f32
    }

    /// Get a total load progress from an iterator of [`LoadProgress`]s
    pub fn merged<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = LoadProgress>,
    {
        let mut loaded = 0;
        let mut total = 0;
        for progress in iter {
            loaded += progress.loaded;
            total += progress.total;
        }

        Self { loaded, total }
    }
}

/// System param containing Bevy resources that may be used to determine load progress
///
/// Currently this only contains the bevy asset server, but this may additionally contain the
/// scripting engine once script loading is implemented.
#[derive(SystemParam)]
pub struct LoadingResources<'w, 's> {
    pub asset_server: Res<'w, AssetServer>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

/// Trait implemented on items that can report their load progress from the [`LoadingResources`].
pub trait HasLoadProgress {
    // Default implementation returns no progress and nothing to load
    fn load_progress(&self, _loading_resources: &LoadingResources) -> LoadProgress {
        LoadProgress::default()
    }
}

// Implement `HasLoadProgress` for asset handles
impl<T: Asset> HasLoadProgress for Handle<T> {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        let loaded = loading_resources.asset_server.get_load_state(self) == LoadState::Loaded;

        LoadProgress {
            loaded: if loaded { 1 } else { 0 },
            total: 1,
        }
    }
}

impl HasLoadProgress for HandleUntyped {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        let loaded = loading_resources.asset_server.get_load_state(self) == LoadState::Loaded;

        LoadProgress {
            loaded: if loaded { 1 } else { 0 },
            total: 1,
        }
    }
}

// Impelement default `HasLoadProgress` for various basic types
macro_rules! impl_default_load_progress {
    ( $($type:ty),* ) => {
        $(
            impl HasLoadProgress for $type {}
        )*
    };
}
impl_default_load_progress!(String, f32, usize, u32, Vec2, Vec3, UVec2, bool, Entity);

#[cfg(feature = "bevy_egui")]
impl_default_load_progress!(bevy_egui::egui::TextureId);

// Implement `HasLoadProgress` for container types
impl<T: HasLoadProgress> HasLoadProgress for Option<T> {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        self.as_ref()
            .map(|x| x.load_progress(loading_resources))
            .unwrap_or_default()
    }
}
impl<T: HasLoadProgress> HasLoadProgress for Vec<T> {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        LoadProgress::merged(self.iter().map(|x| x.load_progress(loading_resources)))
    }
}
impl<K, T: HasLoadProgress> HasLoadProgress for HashMap<K, T> {
    fn load_progress(&self, loading_resources: &LoadingResources) -> LoadProgress {
        LoadProgress::merged(self.values().map(|x| x.load_progress(loading_resources)))
    }
}
