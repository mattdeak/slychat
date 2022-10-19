use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use chatroom::SimpleChatRoom;
use server::Server;
use tokio::net::TcpListener;

mod chatroom;
mod listeners;
mod server;
mod user;

const IP: &str = "127.0.0.1";
const PORT: usize = 9001;

#[tokio::main]
async fn main() {
    let mut server: Server<SimpleChatRoom> = Server::new();

    let address = format!("{}:{}", IP, PORT);
    let listener = TcpListener::bind(address).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        listeners::initialize_connection(socket, &mut server).await;
    }
}
