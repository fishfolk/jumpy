use hecs::World;

pub mod api;

pub fn update_network_client(world: &mut World, delta_time: f32) {
    update_network_common(world, delta_time);
}

pub fn fixed_update_network_client(world: &mut World, delta_time: f32, integration_factor: f32) {
    fixed_update_network_common(world, delta_time, integration_factor);
}

pub fn update_network_host(world: &mut World, delta_time: f32) {
    update_network_common(world, delta_time);
}

pub fn fixed_update_network_host(world: &mut World, delta_time: f32, integration_factor: f32) {
    fixed_update_network_common(world, delta_time, integration_factor);
}

fn update_network_common(_world: &mut World, delta_time: f32) {}

fn fixed_update_network_common(_world: &mut World, delta_time: f32, integration_factor: f32) {}
