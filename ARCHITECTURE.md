# Project Architecture

This project use [Macroquad](https://github.com/not-fl3/macroquad) for rendering and context handling and
[Hecs](https://github.com/Ralith/hecs) as its ECS. The game loop is also handled using Macroquad, with a `Node`
implementation (`crate::game::Game`) that calls the various systems in the appropriate frame phase.

Modules should separate code by context, and we keep both component types and system function implementations
together, separated by concern, based on this context.

The overarching design philosophy, on a code level, will be to keep to the ECS paradigm. Meaning that we implement
logic in systems and keep component types as pure data types. The exception is in cases where manipulating
data on a component depends on several steps and this manipulation happens in more than one step, making a setter
type method on the component type to avoid code duplication.

Types will often have companion types with a `Metadata` suffix, which are the types used to serialize and
deserialize the type to json.

There is also the `assets` folder, which contains all assets, both containing data files and the game's graphics.
Every asset type is usually accompanied by a data file, where the assets are declared for dynamic loading.