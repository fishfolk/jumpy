//! Utilities for asset and meta data handling.

use crate::prelude::*;

/// Error type for failure to resolve [`UntypedHandle`] to metadata asset from  `Handle<T>` or `Handle<ElementMeta>`
/// (where `T` is metadata type).
#[derive(thiserror::Error, Debug, Clone)]
pub enum MetaHandleCastError {
    #[error("UntypedHandle does not represent Handle<{0}> or Handle<ElementMeta>")]
    BadHandleType(String),
    #[error("UntypedHandle maps to Handle<ElementMeta> but ElementMeta inner handle does not cast to {0}")]
    ElementDataMismatch(String),
    #[error("Failed to retrieve asset for handle (required for cast type valdiation)")]
    InvalidHandle,
}

/// Try to get metadata handle from [`UntypedHandle`] that may represent direct handle to meta (`Handle<T>`)
/// or `Handle<ElementMeta>` where [`ElementMeta`]'s data is castable to `Handle<T>`.
///
/// [`Handle::untyped`] can be used to convert to [`UntypedHandle`].
///
/// This is useful for code that wants to spawn an item and take UntypedHandle to allow either direct meta handle
/// or `Handle<ElementMeta>` as argument.
pub fn try_cast_meta_handle<T: HasSchema>(
    handle: UntypedHandle,
    assets: &AssetServer,
) -> Result<Handle<T>, MetaHandleCastError> {
    // Get untyped asset from handle or error on failure
    let asset = assets
        .try_get_untyped(handle)
        .ok_or(MetaHandleCastError::InvalidHandle)?;

    // If asset casts to T, return it as Handle<T>
    if asset.try_cast_ref::<T>().is_ok() {
        return Ok(handle.typed::<T>());
    }

    // Check if handle type is ElementMeta
    if let Ok(element_meta) = asset.try_cast_ref::<ElementMeta>() {
        // Does element data cast to T?
        if assets.get(element_meta.data).try_cast_ref::<T>().is_ok() {
            // Return ElementMeta's data as Handle<T>
            Ok(element_meta.data.untyped().typed())
        } else {
            // ElementMeta data does not cast to T.
            Err(MetaHandleCastError::ElementDataMismatch(
                std::any::type_name::<T>().to_string(),
            ))
        }
    } else {
        // UntypedHandle is neither Handle<T> or Handle<ElementMeta>.
        Err(MetaHandleCastError::BadHandleType(
            std::any::type_name::<T>().to_string(),
        ))
    }
}
