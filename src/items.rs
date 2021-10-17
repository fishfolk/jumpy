//! Things available to spawn from the level editor
//! Proto-mods, eventually some of the items will move to some sort of a wasm runtime

mod cannon;
mod galleon;
mod grenades;
mod gun;
pub mod jellyfish;
mod machine_gun;
mod mines;
mod musket;
mod shark_rain;
pub mod shoes;
mod sniper;
mod sproinger;
mod sword;
mod turtle_shell;
mod volcano;

pub mod effects;

use macroquad::{
    experimental::{
        collections::storage,
        scene::{HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    capabilities::NetworkReplicate,
    components::{PhysicsBody, Sprite, SpriteParams},
    json, GameWorld,
};

use effects::{Effect, EffectDelivery};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemKind {
    Weapon {
        #[serde(flatten)]
        effect: Effect,
        #[serde(flatten)]
        effect_delivery: EffectDelivery,
        cooldown: f32,
        #[serde(default)]
        recoil: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemParams {
    pub id: String,
    #[serde(flatten)]
    pub kind: ItemKind,
    #[serde(with = "json::uvec2_def")]
    pub size: UVec2,
    #[serde(flatten)]
    pub sprite: SpriteParams,
}

pub struct Item {
    pub id: String,
    pub kind: ItemKind,
    pub sprite: Sprite,
    pub body: PhysicsBody,
}

impl Item {
    pub fn new(position: Vec2, params: ItemParams) -> Self {
        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
            position,
            0.0,
            params.size.as_f32(),
        );

        let sprite = Sprite::new(params.sprite);

        Item {
            id: params.id,
            kind: params.kind,
            body,
            sprite,
        }
    }

    fn network_update(mut node: RefMut<Self>) {
        node.body.update();
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Item>();
            Item::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}

impl Node for Item {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        node.sprite.draw(node.body.pos, node.body.angle, None);
    }
}

/// Proto-mod
/// A meta description on how to create an item from the map
pub struct OldItem {
    /// Tiled object name used on the objects layer, like "sword" or "sproinger"
    pub tiled_name: &'static str,
    pub constructor: fn(_: Vec2) -> HandleUntyped,
    /// Spawn offset from a tiled object position
    /// Mostly legacy, should be gone with a proper level editor
    /// may be will be a Vec2 soon, waiting for https://github.com/bitshifter/glam-rs/issues/76
    pub tiled_offset: (f32, f32),
    /// List of audio resources to load
    /// Later they will be accessible in resources.items_sounds
    /// by "tiled_name/resource_id"
    pub sounds: &'static [(&'static str, &'static str)],
    /// List of fxses to load. Each fx will be an EmitterCache, rendered
    /// in the world space
    /// Later they will be accessible in resources.items_fxses
    /// by "tiled_name/resource_id"
    pub fxses: &'static [(&'static str, &'static str)],
    /// Right now items used in network play should be carefull
    /// about using random and similar things
    /// It will be automatically tested and undetermenistic weapons will be denied
    /// by the game itself, but, right now, its up for a weapon developer to veryfy
    /// that nothing network-illegal is going on
    pub network_ready: bool,
}

pub const ITEMS: &[OldItem] = &[
    OldItem {
        tiled_name: "sword",
        constructor: sword::Sword::spawn,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: true,
    },
    OldItem {
        tiled_name: "sproinger",
        constructor: sproinger::Sproinger::spawn,
        tiled_offset: (-35., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: true,
    },
    OldItem {
        tiled_name: "musket",
        constructor: gun::Gun::spawn_musket,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: true,
    },
    OldItem {
        tiled_name: "sniper",
        constructor: gun::Gun::spawn_sniper,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: true,
    },
    OldItem {
        tiled_name: "machine_gun",
        constructor: machine_gun::MachineGun::spawn,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "mines",
        constructor: mines::Mines::spawn,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "cannon",
        constructor: cannon::Cannon::spawn,
        tiled_offset: (-35., -25.),
        sounds: &[],
        fxses: &[],
        network_ready: false, // There's no random but I can't verify)
    },
    OldItem {
        tiled_name: "turtle_shell",
        constructor: turtle_shell::TurtleShell::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "grenade",
        constructor: grenades::Grenades::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "boots",
        constructor: shoes::Shoes::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "volcano",
        constructor: volcano::Volcano::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "galleon",
        constructor: galleon::Galleon::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "jellyfish",
        constructor: jellyfish::Jellyfish::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
    OldItem {
        tiled_name: "shark_rain",
        constructor: shark_rain::SharkRain::spawn,
        tiled_offset: (0., 0.),
        sounds: &[],
        fxses: &[],
        network_ready: false,
    },
];
