use std::any::TypeId;

use once_cell::sync::Lazy;

use crate::prelude::*;

pub mod client;
pub mod proto;
// pub mod server;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, _app: &mut App) {}
}

pub static NET_MESSAGE_TYPES: Lazy<Vec<TypeId>> = Lazy::new(|| {
    [
        TypeId::of::<proto::Ping>(),
        TypeId::of::<proto::Pong>(),
        TypeId::of::<proto::ClientMatchInfo>(),
        TypeId::of::<proto::match_setup::MatchSetupFromClient>(),
        TypeId::of::<proto::match_setup::MatchSetupFromServer>(),
    ]
    .to_vec()
});
