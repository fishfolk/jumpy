use std::collections::HashMap;

use crate::network::message::NetworkMessage;
use async_trait::async_trait;

use crate::Result;

use super::{Id, Lobby, NetworkEvent, Player, RequestStatus};

static mut API_INSTANCE: Option<Api> = None;

pub struct Api {
    backend: Box<dyn ApiBackend>,
}

impl Api {
    fn try_get_instance() -> Option<&'static mut Api> {
        unsafe { API_INSTANCE.as_mut() }
    }

    fn get_instance() -> &'static mut Api {
        Self::try_get_instance()
            .unwrap_or_else(|| panic!("Api::get_instance was called before Api::init"))
    }

    pub async fn init<T: 'static + ApiBackend + Default>(token: &str) -> Result<()> {
        let backend = Box::new(T::default());

        let mut api = Api { backend };

        api.backend.init(token).await?;

        unsafe { API_INSTANCE = Some(api) };

        Ok(())
    }

    pub async fn close() -> Result<()> {
        let api = Self::get_instance();

        api.backend.close().await
    }

    pub fn get_player_id() -> Result<Id> {
        let api = Self::get_instance();

        api.backend.get_player_id()
    }

    pub async fn get_player(id: &Id) -> Result<Player> {
        let api = Self::get_instance();

        api.backend.get_player(id).await
    }

    pub async fn get_lobby(id: &Id) -> Result<Lobby> {
        let api = Self::get_instance();

        api.backend.get_lobby(id).await
    }

    pub fn dispatch_message(message: NetworkMessage) -> Result<()> {
        let api = Self::get_instance();

        api.backend.dispatch_message(message)
    }

    pub fn next_event() -> Option<NetworkEvent> {
        let api = Self::get_instance();

        api.backend.next_event()
    }
}

/// This trait should be implemented by all backend implementations
#[async_trait]
pub trait ApiBackend {
    /// Init backend connect to API
    async fn init(&mut self, token: &str) -> Result<()>;
    /// Close API connection
    async fn close(&mut self) -> Result<()>;
    /// Get the local players id
    fn get_player_id(&self) -> Result<Id>;
    /// Get `Player` with the specified `id`
    async fn get_player(&mut self, id: &Id) -> Result<Player>;
    /// Get `Lobby` with the specified `id`
    async fn get_lobby(&mut self, id: &Id) -> Result<Lobby>;
    /// Dispatch a network message
    fn dispatch_message(&mut self, message: NetworkMessage) -> Result<()>;
    /// Get next event from the queue
    fn next_event(&mut self) -> Option<NetworkEvent>;
}

/// This is used as a placeholder for when no external backend implementation is available.
/// Will be removed once we have a backend that can be freely redistributed (Steam, probably)
#[allow(dead_code)]
pub struct MockApiBackend {
    player_id: Id,
    players: Vec<Player>,
    lobbies: Vec<Lobby>,
    sessions: HashMap<String, Id>,
}

impl MockApiBackend {
    pub fn new() -> Self {
        let player_id = Id::from("1");

        let players = vec![
            Player::new(&player_id, "Player One"),
            Player::new(&Id::from("2"), "Player Two"),
        ];

        let mut sessions = HashMap::new();

        sessions.insert("player_one_token".to_string(), players[0].id.clone());
        sessions.insert("player_two_token".to_string(), players[1].id.clone());

        MockApiBackend {
            player_id,
            players,
            lobbies: Vec::new(),
            sessions,
        }
    }
}

impl Default for MockApiBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ApiBackend for MockApiBackend {
    async fn init(&mut self, _token: &str) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_player_id(&self) -> Result<Id> {
        Ok(self.player_id.clone())
    }

    async fn get_player(&mut self, id: &Id) -> Result<Player> {
        if let Some(player) = self.players.iter().find(|&player| player.id == *id) {
            Ok(player.clone())
        } else {
            Err(RequestStatus::NotFound.into())
        }
    }

    async fn get_lobby(&mut self, id: &Id) -> Result<Lobby> {
        if let Some(lobby) = self.lobbies.iter().find(|&lobby| lobby.id == *id) {
            Ok(lobby.clone())
        } else {
            Err(RequestStatus::NotFound.into())
        }
    }

    fn dispatch_message(&mut self, _message: NetworkMessage) -> Result<()> {
        Ok(())
    }

    fn next_event(&mut self) -> Option<NetworkEvent> {
        None
    }
}
