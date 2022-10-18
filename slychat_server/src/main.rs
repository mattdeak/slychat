use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use chatroom::{ChatRoom, SimpleChatRoom};
use server::Server;
use slychat_common::UserKey;
use tokio::sync::mpsc::Sender;
use tokio::{net::TcpStream, sync::mpsc::Receiver};

mod chatroom;
mod listeners;
mod server;
mod user;

const DEFAULT_CAPACITY: usize = 64;

#[tokio::main]
async fn main() {
    let mut server: Server<SimpleChatRoom> = Server::new();
    server.create_chatroom("test".to_string(), DEFAULT_CAPACITY);
}
