use crate::prelude::*;

pub mod client;
pub mod proto;
// pub mod server;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, _app: &mut App) {}
}
