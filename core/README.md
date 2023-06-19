# Jumpy Core

[`jumpy_core`][crate] contains the core Jumpy game loop. This includes things like physics, the
player controller, the items, and pretty much everything else that happens during the play of a
match in the game.

**If you want to contribute to Jumpy's match gameplay, this is likely where you want to be.**

## Overview

The Jumpy core logic is fully implemented on top of [`bones_lib`][crate::external::bones_lib]
and the Bones ECS ( [`bones_lib::ecs`][crate::external::bones_lib::ecs] ).

By only interacting with `bones_lib`, we isolate the core game from the specifics of rendering and
user input, which is handled by the `jumpy` crate.

Jumpy core is primarily interacted with through the [`CoreSession`][crate::session::CoreSession]
struct, which allows you to:

- Provide user input to the game
- Run a game frame by executing the core systems to update the state of the contained
  [`bones_lib::ecs::World`]
- Snapshot and restore the state of the game

This means that Jumpy core has no interaction with rendering or how to collect the user input,
keeping it focused just on the core game mechanics.

Jumpy core is also designed to be deterministic, lending it well to rollback networking, as implemented in the `jumpy` crate.

### Important Concepts

#### Gameplay Modules

The gameplay is split into different Rust modules for different aspects of the game. For instance,
we have the [`damage`] that handles damage regions and the [`camera`] module with the camera
controller.

The modules that contain gameplay systems all have an `install()` function that adds it's systems to
the [`CoreSession`][crate::session::CoreSession].

See [`install_modules()`] for more details on how to add new systems and modules.

#### Game Metadata

Metadata in Jumpy core is any data that is loaded as an asset at runtime. Examples of assets
include:

- Sprites
- Maps
- Player animations
- Item data

Most of our metadata is loaded from YAML files, with the format being defined by Rust structs that
derive `Serialize` and `Deserialize`.

See the [`metadata`] module for more details.

#### Entity Hydration

Throughout [`jumpy_core`][crate] you'll see `hydrate` functions. The goal of these functions is to
take "stub entities" that have only some of the components that they need, and then "hydrate" them
by adding all of the remaining components that they need to work properly.

For example, the [`player_spawner`][crate::elements::player_spawner] spawns players by creating an
entity with only a [`PlayerIdx`][crate::player::PlayerIdx] and
[`Transform`][crate::prelude::Transform] component. This creates a "player stub" that is missing all
kinds of important components such as the sprite, collisions, etc.

Later the [`hydrate_players()`][crate::player] system will run, find all of the player stubs, and
lookup from the [`PlayerIdx`][crate::player::PlayerIdx] what sprites need to be added, etc. before
adding all the required components to the entity.

This practice of hydrating entity stubs makes it much simpler to spawn different kinds of entities
throughout the codebase, without needing to duplicate the much more complicated logic of adding all
of that entities required components.

## 🚧 Work-in-Progress Bones Asset Handling Abstraction

`bones_lib` is designed to be independent of Bevy, such that you could possibly make a renderer for
any `bones_lib` core in any game engine you wanted, not just Bevy. This means that, ideally,
`jumpy_core` would have no dependency on `bevy`, but right now that is not completely true.

`bones_lib` is still working out how to implement it's own asset abstraction, and until it gets
fully figured out, we do have a dependency on Bevy through the
[`BonesBevyAsset`][crate::external::bones_bevy_asset::BonesBevyAsset] derive macro and some
interactions necessary to access Bevy assets in the [`CoreSession`][crate::session::CoreSession].
