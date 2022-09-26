use bevy_fluent::BundleAsset;
use unic_langid::LanguageIdentifier;

use super::*;

#[derive(HasLoadProgress, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TranslationsMeta {
    /// The locale setting detected on the user's system
    #[serde(skip)]
    #[has_load_progress(none)]
    pub detected_locale: LanguageIdentifier,
    /// The default locale that will be used if a message is not found in the user's selected locale
    #[has_load_progress(none)]
    pub default_locale: LanguageIdentifier,
    /// Paths to the locale resources to load
    pub locales: Vec<String>,
    /// The handles to the locale bundle assets
    #[serde(skip)]
    pub locale_handles: Vec<Handle<BundleAsset>>,
}
