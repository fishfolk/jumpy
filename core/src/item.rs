use crate::prelude::*;

/// Marker component for items.
///
/// Items are any entity that players can pick up and use.
#[derive(Clone, Copy, TypeUlid)]
#[ulid = "01GP4DBSEB3R6ZNBNNTSY36GW4"]
pub struct Item;

/// An intventory component, indicating another entity that the player is carrying.
#[derive(Clone, TypeUlid, Default, Deref, DerefMut)]
#[ulid = "01GP4D6M2QBSKZMEZMM22YGG41"]
pub struct Inventory(pub Option<Entity>);

/// A helper struct containing a player-inventory pair that indicates the given player is holding
/// the other entity in their inventory.
#[derive(Debug, Clone, Copy)]
pub struct Inv {
    pub player: Entity,
    pub inventory: Entity,
}

/// System param that can be used to conveniently get the inventory of each player.
#[derive(Deref, DerefMut, Debug)]
pub struct PlayerInventories<'a>(&'a [Option<Inv>; MAX_PLAYERS]);

impl<'a> SystemParam for PlayerInventories<'a> {
    type State = [Option<Inv>; MAX_PLAYERS];
    type Param<'s> = PlayerInventories<'s>;

    fn initialize(world: &mut World) {
        world.components.init::<Inventory>();
        world.components.init::<PlayerIdx>();
    }

    fn get_state(world: &World) -> Self::State {
        world
            .run_initialized_system(
                |entities: Res<Entities>,
                 player_indexes: Comp<PlayerIdx>,
                 inventories: Comp<Inventory>| {
                    let mut player_inventories = [None; MAX_PLAYERS];
                    for (player, (idx, inventory)) in
                        entities.iter_with((&player_indexes, &inventories))
                    {
                        if let Some(inventory) = inventory.0 {
                            player_inventories[idx.0] = Some(Inv { player, inventory });
                        }
                    }

                    Ok(player_inventories)
                },
            )
            .unwrap()
    }

    fn borrow<'s>(state: &mut Self::State) -> Self::Param<'_> {
        PlayerInventories(state)
    }
}

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
