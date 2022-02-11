//! This module holds the networking core, used

use hecs::World;

use core::network::Api;
use core::Result;

#[cfg(feature = "ultimate")]
pub async fn init_api(token: &str) -> Result<()> {
    Api::init::<ultimate::UltimateApiBackend>(token).await
}

#[cfg(not(feature = "ultimate"))]
pub async fn init_api(token: &str) -> Result<()> {
    Api::init::<core::network::MockApiBackend>(token).await
}

#[derive(Default)]
pub struct NetworkClient {}

impl NetworkClient {
    pub fn new() -> Self {
        NetworkClient {}
    }
}

pub fn update_network_client(_world: &mut World) {
    /*
        Api::dispatch_message(NetworkMessage::PlayerInput {
            player_id: player_id.clone(),
            input,
        });
    */
    while let Some(_event) = Api::next_event() {}
}

pub fn fixed_update_network_client(_world: &mut World) {}

#[derive(Default)]
pub struct NetworkHost {}

impl NetworkHost {
    pub fn new() -> Self {
        NetworkHost {}
    }
}

pub fn update_network_host(_world: &mut World) {
    while let Some(_event) = Api::next_event() {}
}

pub fn fixed_update_network_host(_world: &mut World) {}
