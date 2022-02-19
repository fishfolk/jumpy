//! This module holds the networking core, used

use hecs::World;

pub fn update_network_client(world: &mut World) {
    update_network_common(world);
}

pub fn fixed_update_network_client(world: &mut World) {
    fixed_update_network_common(world);
}

pub fn update_network_host(world: &mut World) {
    update_network_common(world);
}

pub fn fixed_update_network_host(world: &mut World) {
    fixed_update_network_common(world);
}

fn update_network_common(_world: &mut World) {}

fn fixed_update_network_common(_world: &mut World) {}
