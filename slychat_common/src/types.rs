use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_serde::{Deserializer, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserKey {
    pub user: String,
    pub public: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response<T> {
    Success(T),
    Error(String),
}

pub trait APICommand: Serialize + DeserializeOwned {}

// Client Side
#[derive(Serialize, Deserialize, Debug)]
pub enum APIRequest {
    LoginRequest(UserKey),
    RefreshRoomKeysRequest,
    SendMessageRequest(String, Vec<u8>),
    ListRoomsRequest,
    JoinRoomRequest(String),
    LeaveRoom,
    Logout,
}

impl APICommand for APIRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub enum APIResponse {
    LoginResponse(Response<()>),
    RefreshRoomKeysResponse(Response<Vec<UserKey>>),
    SendMessageResponse(Response<()>),
    PublishMessage(Vec<u8>),
    ListRoomsResponse(Response<Vec<String>>),
    JoinRoomResponse(Response<()>),
}

impl APICommand for APIResponse {}
