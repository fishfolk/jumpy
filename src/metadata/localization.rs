use bevy_fluent::BundleAsset;
use unic_langid::LanguageIdentifier;

use super::*;

#[derive(BonesBevyAssetLoad, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TranslationsMeta {
    // TODO: Custom implementation of BonesBevyAssetLoad that will detect the system locale with the sys-locale crate.
    /// The locale setting detected on the user's system
    #[serde(skip)]
    #[asset(deserialize_only)]
    pub detected_locale: LanguageIdentifier,
    /// The default locale that will be used if a message is not found in the user's selected locale
    #[asset(deserialize_only)]
    pub default_locale: LanguageIdentifier,
    /// The handles to the locale bundle assets
    pub locales: Vec<AssetHandle<BundleAsset>>,
}
