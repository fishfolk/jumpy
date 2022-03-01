use hecs::World;

pub mod api {
    pub use ultimate::UltimateApi as Api;
}

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
