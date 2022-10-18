use serde::{Deserialize, Serialize};

pub mod encryption;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserKey {
    pub user: String,
    pub public: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum APICommand {
    Greet(UserKey),
    RefreshRoomKeysRequest,
    RefreshRoomKeysResponse(Vec<UserKey>),
    GoodBye,
    Message(String, Vec<u8>),
}
#[cfg(test)]
mod tests {
    use super::*;
}
