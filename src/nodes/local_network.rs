//! Node that emulates network for local play
//! This is temporary solution, in the future "LocalNetwork" node should
//! be able to mix local/remote players
//!
//! But, right now, with fixed-delay networking
//! it is nice to run completely local, no-delay game

use macroquad::experimental::scene::{self, Handle, Node, NodeWith, RefMut};

use crate::{capabilities::NetworkReplicate, input, nodes::Player};

pub struct LocalNetwork {
    player1_input: input::InputScheme,
    player1: Handle<Player>,
    player2_input: input::InputScheme,
    player2: Handle<Player>,

    paused: bool,
}

impl LocalNetwork {
    pub fn new(
        players_input: Vec<input::InputScheme>,
        player1: Handle<Player>,
        player2: Handle<Player>,
    ) -> LocalNetwork {
        assert!(players_input.len() == 2);
        LocalNetwork {
            player1,
            player2,
            player1_input: players_input[0],
            player2_input: players_input[1],
            paused: false,
        }
    }
}

impl Node for LocalNetwork {
    fn fixed_update(mut node: RefMut<Self>) {
        scene::get_node(node.player1).apply_input(input::collect_input(node.player1_input));
        scene::get_node(node.player2).apply_input(input::collect_input(node.player2_input));

        if macroquad::input::is_key_down(macroquad::prelude::KeyCode::Z) {
            node.paused = true;
        }
        if macroquad::input::is_key_down(macroquad::prelude::KeyCode::X) {
            node.paused = false;
        }

        if !node.paused {
            for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
                (capability.network_update)(node);
            }
        }
    }
}
