use serde::{Deserialize, Serialize};

pub mod encryption;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserKey {
    pub user: String,
    pub public: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerRequest {
    Greet(UserKey),
    RefreshRoomKeys,
    GoodBye,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UserMessage {
    Message(String, Vec<u8>),
    Command(ServerRequest),
}

#[cfg(test)]
mod tests {
    use super::*;
}
