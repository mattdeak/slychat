use chatroom::{ChatRoom, SimpleChatRoom};
use log::LevelFilter;
use server::Server;
use simple_logger::SimpleLogger;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

mod chatroom;
mod listeners;
mod server;

const IP: &str = "127.0.0.1";
const PORT: usize = 9001;

type ServerMutex<G: ChatRoom> = Arc<Mutex<Server<G>>>;
use log::{Level, Metadata, Record};

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    let (sx, rx) = tokio::sync::mpsc::channel(64);
    let server: ServerMutex<SimpleChatRoom> = Arc::new(Mutex::new(Server::build(rx)));

    let address = format!("{}:{}", IP, PORT);
    let listener = TcpListener::bind(address).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let s = server.clone();
        tokio::spawn(async move { listeners::process(socket, s).await });
    }
}
