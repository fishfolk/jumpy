use std::borrow::Borrow;

use bevy::{prelude::*, utils::HashMap};
use bevy_fluent::prelude::*;
use fluent::FluentArgs;
use fluent_content::{Content, Request};

/// Plugin for initializing and loading the [`Localization`] resource.
pub struct JumpyLocalizationPlugin;

impl Plugin for JumpyLocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FluentPlugin)
            .init_resource::<Locale>()
            .insert_resource(Localization::new());

        app.add_system(load_locales);
    }
}

/// Extension trait to reduce boilerplate when getting values from a [`Localization`].
pub trait LocalizationExt<'a, T: Into<Request<'a, U>>, U: Borrow<FluentArgs<'a>>> {
    /// Request message content and get an empty string if it doesn't exist.
    fn get(&self, request: T) -> String;
}

impl<'a, T, U> LocalizationExt<'a, T, U> for Localization
where
    T: Copy + Into<Request<'a, U>> + std::fmt::Debug,
    U: Borrow<FluentArgs<'a>>,
{
    /// Request message content and get an empty string if it doesn't exist.
    fn get(&self, request: T) -> String {
        let response = self.content(request);

        if response.is_none() {
            debug!(
                "Missing response for {request:?}. \
                ( Could be normal if localization is still loading )"
            );
        }

        response.unwrap_or_default()
    }
}

/// Watch for locale [`BundleAsset`] load events and add any new bundles to the [`Localization`]
/// resource.
fn load_locales(
    locale: Res<Locale>,
    mut loading_bundles: Local<Vec<Handle<BundleAsset>>>,
    mut localization: ResMut<Localization>,
    mut events: EventReader<AssetEvent<BundleAsset>>,
    assets: Res<Assets<BundleAsset>>,
) {
    // Whether there was an update that means we need to rebuild the localization
    let mut should_rebuild_localization = false;

    // We need to reload the localization if our Locale changed
    if locale.is_changed() {
        should_rebuild_localization = true;
    }

    // Collect any previously loading locales that we should try to load again
    let mut try_to_load = loading_bundles.drain(..).collect::<Vec<_>>();

    // Add any updated or created assets to the list of bundles to try to load
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                try_to_load.push(handle.clone_weak());
            }
            _ => (),
        }
    }

    // Try to load all bundles that need to be loaded
    for handle in try_to_load {
        // If the asset is loaded now
        if assets.get(&handle).is_some() {
            // We need to rebuild the localization
            should_rebuild_localization = true;

        // If it isn't loaded yet
        } else {
            // Try to load it next frame
            loading_bundles.push(handle);
        }
    }

    // Rebuild localization if anything was updated
    if should_rebuild_localization {
        let mut new_localization = Localization::new();

        // Create map containing the bundles and their handles, indexed by their locale
        let mut bundles = assets
            .iter()
            .map(|(handle_id, asset)| (&asset.locales[0], (handle_id, asset)))
            .collect::<HashMap<_, _>>();

        // Extract sorted list of locales in a fallback chain
        let locales = locale.fallback_chain(bundles.keys().cloned());

        // Insert bundles into the localization in sorted order
        for locale in locales {
            if let Some((handle_id, bundle)) = bundles.remove(locale) {
                new_localization.insert(&Handle::weak(handle_id), bundle);
            }
        }

        // Update the localization resource
        *localization = new_localization;
    }
}
