use macroquad::experimental::scene::{Handle, Node, RefMut};
use macroquad::prelude::scene::NodeWith;
use macroquad::prelude::*;

use crate::resources::MapResource;
use crate::network::{AccountId, Api, Client, Server, DEFAULT_SERVER_PORT};
use crate::player::{PlayerControllerKind, PlayerParams};
use crate::{collect_local_input, GameInput, Result};

pub struct NetworkGame {
    players: Vec<(PlayerParams, Handle<OldPlayer>)>,
    is_host: bool,
}

impl NetworkGame {
    pub fn new(
        host_id: AccountId,
        _map_resource: MapResource,
        players: &[PlayerParams],
    ) -> Result<Self> {
        let is_host = Api::get_instance().is_own_id(host_id)?;

        if is_host {
            let server = Server::new(DEFAULT_SERVER_PORT, players)?;
            scene::add_node(server);
        } else {
            let client = Client::new(host_id)?;
            scene::add_node(client);
        }

        let players = Vec::new();

        let res = NetworkGame { players, is_host };

        Ok(res)
    }

    pub fn apply_player_input(&mut self, index: u8, input: GameInput) {
        let res = self
            .players
            .iter()
            .find(|(p, _)| p.index == index)
            .map(|(_, handle)| handle);

        if let Some(handle) = res {
            let mut node = scene::get_node(*handle);
            node.input = input;
        }
    }
}

impl Node for NetworkGame {
    fn update(node: RefMut<Self>)
    where
        Self: Sized,
    {
        for (params, handle) in &node.players {
            if let PlayerControllerKind::LocalInput(input_scheme) = params.controller {
                let mut node = scene::get_node(*handle);
                node.input = collect_local_input(input_scheme);
            }
        }
    }

    fn fixed_update(node: RefMut<Self>)
    where
        Self: Sized,
    {
    }
}
