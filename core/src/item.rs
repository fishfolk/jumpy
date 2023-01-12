use crate::prelude::*;

/// Marker component for items.
///
/// Items are any entity that players can pick up and use.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GP4DBSEB3R6ZNBNNTSY36GW4"]
pub struct Item;

/// Marker component added to items when they are dropped.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GP4DH23M7M2CXVWADPZHW54F"]
pub struct ItemDropped {
    /// The player that dropped the item
    pub player: Entity,
}

/// Marker component added to items when they are grabbed.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GP4DJ2RPYTDPKSKEK8JKK9VT"]
pub struct ItemGrabbed;

/// Marker component added to items when they are used.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GP4DJ84TFB8Z7H9VY7Y0R47H"]
pub struct ItemUsed;
