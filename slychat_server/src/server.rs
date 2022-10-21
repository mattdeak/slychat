use bimap::BiMap;
use slychat_common::types::UserKey;
use std::{collections::HashMap, error::Error, fmt::Display, hash::Hash};

use tokio::{io::AsyncWriteExt, sync::mpsc::Receiver};

use crate::chatroom::{self, ChatRoom, ChatRoomError};

#[derive(Debug, Clone)]
pub enum ServerError {
    UserError(String),
    ChatRoomError(ChatRoomError),
    InvalidChatRoomError,
}

impl From<ChatRoomError> for ServerError {
    fn from(e: ChatRoomError) -> Self {
        Self::ChatRoomError(e)
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::UserError(s) => write!(f, "{}", s),
            ChatRoomError => ChatRoomError.fmt(f),
        }
    }
}

impl Error for ServerError {}

const DEFAULT_CAPACITY: usize = 64;
const WAITING_ROOM: &str = "waiting";

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct UserId(String);

impl<S> From<S> for UserId
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ChatRoomId(String);

impl<S> From<S> for ChatRoomId
where
    S: Into<String>,
{
    fn from(s: S) -> Self {
        Self(s.into())
    }
}
// I don't know what the fuck to do because
// Accessing the users which need to mutate often because socket needs to be mutable
// Is terrible! Terrible!
pub struct Server<G: ChatRoom> {
    // Public key registry
    pub key_registry: HashMap<UserId, Vec<u8>>,
    pub receiver: Receiver<String>,
    pub user_handlers: HashMap<UserId, Receiver<String>>,
    pub chat_rooms: HashMap<ChatRoomId, G>,
    pub chatroom_registry: BiMap<UserId, ChatRoomId>,
}

impl<G: ChatRoom> Server<G> {
    pub fn build(receiver: Receiver<String>) -> Self {
        let wr_str = WAITING_ROOM.to_string();

        let mut server = Self {
            receiver,
            key_registry: HashMap::new(),
            user_handlers: HashMap::new(),
            chat_rooms: HashMap::new(),
            chatroom_registry: BiMap::new(),
        };

        server
            .create_chatroom(wr_str, DEFAULT_CAPACITY)
            .expect("Failed to create waiting room during server build.");

        // Create waiting room
        server
    }

    pub async fn receive_loop(&mut self) {
        loop {
            if let Some(x) = self.receiver.recv().await {
                todo!()
            }
        }
    }

    pub fn register_user(
        &mut self,
        mut user: &String,
        mut sender: tokio::sync::mpsc::Sender<String>,
        public: Vec<u8>,
    ) -> Result<(), ServerError> {
        // Should do a membership check first
        if self.user_handlers.contains_key(&UserId(user.to_string())) {
            return Err(ServerError::UserError(
                "User already registered.".to_string(),
            ));
        }

        self.chat_rooms
            .get_mut(&WAITING_ROOM.into())
            .expect("Could not find waiting room")
            .register_user(user, public)?;

        Ok(())
    }

    pub fn create_chatroom(&mut self, chatroom_name: String, capacity: usize) -> Result<&G, &str> {
        let chatroom_key: ChatRoomId = chatroom_name.clone().into();
        if self.chat_rooms.contains_key(&chatroom_key) {
            return Err("Chatroom could not be created.");
        }

        let chatroom = ChatRoom::build(chatroom_name, capacity);
        self.chat_rooms.insert(chatroom_key.clone(), chatroom);

        // Probably inefficient
        Ok(&self.chat_rooms[&chatroom_key])
    }

    pub fn delete_chatroom(&mut self, chatroom_name: String) -> Result<(), &str> {
        let chatroom_key: ChatRoomId = chatroom_name.into();
        if self.chat_rooms.contains_key(&chatroom_key) {
            self.chat_rooms.remove(&chatroom_key);
            return Ok(());
        } else {
            return Err("Chatroom not found.");
        }
    }

    pub fn get_active_room(&self, username: &String) -> Result<&ChatRoomId, ServerError> {
        let user_key: UserId = username.into();
        if let Some(c) = self.chatroom_registry.get_by_left(&user_key) {
            Ok(c)
        } else {
            Err(ServerError::InvalidChatRoomError)
        }
    }

    async fn send_roomkeys<W: AsyncWriteExt + Unpin + Send + 'static>(&self, socket: &W) {}
}
