use std::{collections::HashMap, error::Error, fmt::Display, hash::Hash};

use slychat_common::UserKey;
use tokio::io::AsyncWriteExt;

use crate::{
    chatroom::{ChatRoom, ChatRoomError},
    user::ChatUser,
};

#[derive(Debug, Clone)]
pub enum ServerError {
    UserError(String),
    ChatRoomError(ChatRoomError),
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
pub struct Server<G: ChatRoom> {
    // I think this will need to be cleverly synchronized
    pub users: Vec<ChatUser>,
    // chat_rooms: HashMap<String, ChatRoom<'static>>,
    pub chat_rooms: HashMap<String, G>,
}

impl<G: ChatRoom> Server<G> {
    pub fn new() -> Self {
        let wr_str = WAITING_ROOM.to_string();

        let mut chat_rooms = HashMap::new();

        chat_rooms.insert(wr_str.clone(), G::build(wr_str, DEFAULT_CAPACITY));
        Self {
            users: Vec::new(),
            chat_rooms,
        }
    }

    pub fn create_user(&mut self, mut user: ChatUser) -> Result<&mut ChatUser, ServerError> {
        // Should do a membership check first
        if !self
            .users
            .iter()
            .any(|cu| user.user_data.user == cu.user_data.user)
        {
            if let Err(e) = self
                .chat_rooms
                .get_mut(&WAITING_ROOM.to_string())
                .expect("Could not find waiting room")
                .register_user(&mut user)
            {
                return Err(e.into());
            };

            self.users.push(user);
            Ok(self.users.last_mut().unwrap())
        } else {
            Err(ServerError::UserError("User already exists.".to_string()))
        }
    }

    pub fn create_chatroom(&mut self, chatroom_name: String, capacity: usize) -> Result<&G, &str> {
        if self.chat_rooms.contains_key(&chatroom_name) {
            return Err("Chatroom could not be created.");
        }

        let chatroom = ChatRoom::build(chatroom_name.clone(), capacity);
        self.chat_rooms.insert(chatroom_name.clone(), chatroom);

        // Probably inefficient
        Ok(&self.chat_rooms[&chatroom_name])
    }

    pub fn delete_chatroom(&mut self, chatroom_name: String) -> Result<(), &str> {
        if self.chat_rooms.contains_key(&chatroom_name) {
            self.chat_rooms.remove(&chatroom_name);
            return Ok(());
        } else {
            return Err("Chatroom not found.");
        }
    }

    pub async fn refresh_roomkeys(&self, username: &String) {
        if let Some(user) = self.users.iter().find(|u| &u.user_data.user == username) {
            self.send_roomkeys(&user.socket).await;
        }
    }

    async fn send_roomkeys<W: AsyncWriteExt + Unpin + Send + 'static>(&self, socket: &W) {}
}
