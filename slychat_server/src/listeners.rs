use std::fmt::Display;

use log::info;
use slychat_common::transport::{read_command, send_command, TransportError};
use slychat_common::types::{APICommand, APIRequest, APIResponse, Response, UserKey};
use tokio::select;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

use crate::server::{UserId, UserMessage};
use crate::ServerMutex;
use crate::{
    chatroom::ChatRoom,
    server::{self, Server},
};

#[derive(Debug, Clone)]
pub enum ListenerError {
    Transport(TransportError),
    Error(&'static str),
}

impl Display for ListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "{}", e),
            Self::Error(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for ListenerError {}

pub async fn process<G: ChatRoom>(
    socket: TcpStream,
    server: ServerMutex<G>,
) -> Result<(), ListenerError> {
    let (mut reader, mut writer) = tokio::io::split(socket);

    let (sender, mut receiver) = tokio::sync::mpsc::channel(64);

    // Handle greeting from socket
    let key = wait_for_greeting(&mut reader, &writer).await?;

    if register_user(&key.user, key.public, &mut writer, sender, &server)
        .await
        .is_err()
    {
        eprintln!("Registration Failed");
    }

    // Start main loop
    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        buffer.clear();

        select! {
            data = read_command(&mut reader) => {
                match process_socket_read(data,  &key.user, &server) {
                    Ok(SocketReadHandle::Response(r)) => {
                        if send_command(&mut writer, &r).await.is_err() {
                        eprintln!("Error encoding command: {:?}", r)
                        }
                    },
                    Ok(_) => {println!("Logout Requested. Logging out safely."); break}
                    Err(_) => {eprintln!("Error occurred during socket read. Forcing logout");break},
                }
            },
            message_data = receiver.recv() => {
                if let Some(message) = message_data {
                    // Format message and send to socket
                    assert!(message.user_id == key.user);

                    let response = APIResponse::PublishMessage(message.message);
                    if send_command(&mut writer, &response).await.is_err() {
                    eprintln!("Error encoding command: {:?}", response)
                    }
                }
            }
            // data = test = receiver.recv() {

            // }

        };
    }
    // TODO: Unregister user here.

    Ok(())
}

enum SocketReadHandle {
    Response(APIResponse),
    Logout,
}

impl From<APIResponse> for SocketReadHandle {
    fn from(s: APIResponse) -> Self {
        Self::Response(s)
    }
}

fn process_socket_read<G: ChatRoom>(
    socket_input: Result<APIRequest, TransportError>,
    user: &String,
    server: &ServerMutex<G>,
) -> Result<SocketReadHandle, &'static str> {
    match socket_input {
        Ok(command) => match command {
            APIRequest::LoginRequest(_) => Ok(APIResponse::LoginResponse(Response::Error(
                "Already logged in.".to_string(),
            ))
            .into()),
            APIRequest::Logout => Ok(SocketReadHandle::Logout),
            APIRequest::RefreshRoomKeysRequest => {
                // TODO: this is a bit messy. Probably better to move this into a function.
                let s = server.lock().unwrap();
                let resp = match s.get_active_room(user) {
                    Ok(active_room) => {
                        match s.chat_rooms.get(active_room).unwrap().get_roomkeys() {
                            Ok(v) => Response::Success(
                                v.iter()
                                    .map(|(user, key)| UserKey {
                                        user: (*user).clone(),
                                        public: key.to_vec(),
                                    })
                                    .collect(),
                            ),
                            _ => Response::Error("Unable to get roomkeys".to_string()),
                        }
                    }
                    Err(_) => Response::Error("Invalid Chatroom".to_string()),
                };
                Ok(APIResponse::RefreshRoomKeysResponse(resp).into())
            }
            APIRequest::SendMessageRequest(_, _) => todo!(),
            APIRequest::ListRoomsRequest => todo!(),
            APIRequest::JoinRoomRequest(_) => todo!(),
            APIRequest::LeaveRoom => todo!(),
        },
        Err(e) => {
            eprintln!("Unable to process last read.");
            Err("Unable to process last read.")
        }
    }
}

async fn wait_for_greeting(
    reader: &mut ReadHalf<TcpStream>,
    writer: &WriteHalf<TcpStream>,
) -> Result<UserKey, ListenerError> {
    match read_command(reader).await {
        Ok(command) => match command {
            APIRequest::LoginRequest(user_key) => {
                println!("Found User: {}", &user_key.user);
                Ok(user_key)
            }
            _ => Err(ListenerError::Error(
                "Expected greeting, got different command",
            )),
        },
        Err(e) => Err(ListenerError::Transport(e)),
    }
}

async fn register_user<G: ChatRoom>(
    user: &String,
    public_key: Vec<u8>,
    writer: &mut WriteHalf<TcpStream>,
    sender: tokio::sync::mpsc::Sender<UserMessage>,
    server_mutex: &ServerMutex<G>,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        let mut server = server_mutex.lock().unwrap();
        server.register_user(user, sender, public_key)?;
    }

    match send_command(writer, &APIResponse::LoginResponse(Response::Success(()))).await {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(ListenerError::Transport(e)))?,
    }
}

#[cfg(test)]
mod tests {}
