use std::borrow::Cow;

use serde::{de::DeserializeSeed, Serialize};

use crate::{
    platform::{Storage, StorageItem},
    prelude::*,
};

/// Global settings, stored and accessed through [`crate::platform::Storage`]
#[derive(HasSchema, Debug, Clone, Default)]
#[repr(C)]
pub struct Settings {
    /// The player controller bindings
    pub player_controls: PlayerControlMethods,
    /// The address of the matchmaking server to connect to for online games.
    pub matchmaking_server: String,
}

impl<'de> Deserialize<'de> for Settings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = SchemaDeserializer(Self::schema()).deserialize(deserializer)?;
        Ok(value.into_inner())
    }
}

impl Serialize for Settings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SchemaSerializer(self.as_schema_ref()).serialize(serializer)
    }
}

impl StorageItem for Settings {
    const STORAGE_KEY: &'static str = "settings";
}

impl Settings {
    /// The key used to store the settings in the [`crate::platform::Storage`] resource.
    pub fn get_stored_or_default<'w>(
        game: &'w GameMeta,
        storage: &'w mut Storage,
    ) -> Cow<'w, Self> {
        if let Some(settings) = storage.get::<Self>() {
            Cow::Owned(settings)
        } else {
            Cow::Borrowed(&game.default_settings)
        }
    }
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerControlMethods {
    /// Controls for game remotes
    pub gamepad: PlayerControlSetting,
    /// Controls for keyboard player 1
    pub keyboard1: PlayerControlSetting,
    /// Controls for keyboard player 2
    pub keyboard2: PlayerControlSetting,
}

/// Binds inputs to player actions
#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerControlSetting {
    pub movement: VirtualDPad,
    pub jump: InputKind,
    pub grab: InputKind,
    pub shoot: InputKind,
    pub slide: InputKind,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct VirtualDPad {
    pub up: InputKind,
    pub down: InputKind,
    pub left: InputKind,
    pub right: InputKind,
}

#[derive(HasSchema, Clone, Debug)]
#[repr(C, u8)]
pub enum InputKind {
    Button(GamepadButton),
    AxisPositive(GamepadAxis),
    AxisNegative(GamepadAxis),
    Keyboard(KeyCode),
}

impl Default for InputKind {
    fn default() -> Self {
        Self::Keyboard(KeyCode::Space)
    }
}
