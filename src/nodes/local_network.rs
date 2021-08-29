//! Node that emulates network for local play
//! This is temporary solution, in the future "LocalNetwork" node should
//! be able to mix local/remote players
//!
//! But, right now, with fixed-delay networking
//! it is nice to run completely local, no-delay game

use macroquad::experimental::scene::{self, Handle, Node, NodeWith, RefMut};

use crate::{
    capabilities::NetworkReplicate,
    input::{self, Input},
    nodes::Player,
};

pub struct LocalNetwork {
    player1: Handle<Player>,
    player2: Handle<Player>,
}

impl LocalNetwork {
    pub fn new(player1: Handle<Player>, player2: Handle<Player>) -> LocalNetwork {
        LocalNetwork { player1, player2 }
    }
}

impl Node for LocalNetwork {
    fn fixed_update(mut node: RefMut<Self>) {
        scene::get_node(node.player1).apply_input(input::collect_input(0));
        scene::get_node(node.player2).apply_input(input::collect_input(1));

        for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
            (capability.network_update)(node);
        }
    }
}
