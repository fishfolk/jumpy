#[derive(Debug, Clone)]
pub struct PlayerId(String);

impl From<PlayerId> for String {
    fn from(id: PlayerId) -> Self {
        id.0
    }
}

impl From<String> for PlayerId {
    fn from(str: String) -> Self {
        PlayerId(str)
    }
}