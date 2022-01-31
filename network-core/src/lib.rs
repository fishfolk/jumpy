mod backend;
mod error;

pub use backend::{Backend, MockBackend};
pub use error::{Error, Result};

use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 9000;

pub type AccountId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: AccountId,
    pub display_name: String,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password_hash: Option<String>,
}

impl Account {
    #[must_use]
    pub fn remove_secrets(self) -> Self {
        Account {
            password_hash: None,
            ..self
        }
    }
}

pub type LobbyId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: LobbyId,
    pub host: AccountId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clients: Vec<AccountId>,
}
