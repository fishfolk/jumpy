use crate::prelude::*;
use smallvec::SmallVec;

use crate::AssetHandle;

pub struct Matchmaker {}

pub trait MatchProvider {
    fn status(&self) -> &MatchmakerStatus;
    fn search_for_match(&mut self, info: SearchMatchInfo);
    fn get_session(&mut self) -> Box<dyn MatchmakerSession>;
    fn cancel(&mut self);
}

pub trait MatchmakerSession {
    fn recv_messages(&mut self) -> SmallVec<[MatchmakerSessionMessage; 5]>;
    fn send_message(&mut self, message: MatchmakerSessionMessageKind);
}

pub enum MatchmakerStatus {
    Idle,
    SearchingForMatch(SearchMatchInfo),
    MatchFound(ClientMatchInfo),
}

pub struct SearchMatchInfo {
    pub room_id: String,
    pub player_count: usize,
}

pub struct ClientMatchInfo {
    pub player_idx: usize,
    pub player_count: usize,
    pub random_seed: usize,
}

pub enum MatchmakerSessionMessageKind {
    SelectPlayer(AssetHandle<PlayerMeta>),
    ConfirmPlayerSelection(bool),
    SelectMap(AssetHandle<MapMeta>),
    Ggrs(ggrs::Message),
}

pub struct MatchmakerSessionMessage {
    pub from_player_idx: usize,
    pub kind: MatchmakerSessionMessageKind,
}
