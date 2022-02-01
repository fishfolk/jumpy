//! This module holds the networking core, used

use hecs::World;

pub use network_core::{Api, Id, Lobby, Player, DEFAULT_PORT};

use crate::Result;

pub async fn init_api(token: &str) -> Result<()> {
    #[cfg(feature = "ultimate")]
    Api::init::<ultimate::UltimateApiBackend>(token).await?;

    #[cfg(not(feature = "ultimate"))]
    Api::init::<network_core::MockApiBackend>(token).await?;

    Ok(())
}

#[allow(dead_code)]
#[derive(Default)]
pub struct NetworkClient {}

impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient {}
    }
}

pub fn update_network_client(_world: &mut World) {}

pub fn fixed_update_network_client(_world: &mut World) {}

#[allow(dead_code)]
#[derive(Default)]
pub struct NetworkHost {}

impl NetworkHost {
    pub fn new() -> Self {
        NetworkHost {}
    }
}

pub fn update_network_host(_world: &mut World) {}

pub fn fixed_update_network_host(_world: &mut World) {}
