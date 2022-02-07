//! This module holds the networking core, used

use hecs::World;

use core::network::Api;
use core::Result;

pub async fn init_api(token: &str) -> Result<()> {
    #[cfg(feature = "ultimate")]
    Api::init::<ultimate::UltimateApiBackend>(token).await?;

    #[cfg(not(feature = "ultimate"))]
    Api::init::<core::network::MockApiBackend>(token).await?;

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
