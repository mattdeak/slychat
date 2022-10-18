use bytes::BytesMut;
use slychat_common::encryption::{decrypt, encrypt, KeyData};
use slychat_common::{APICommand, UserKey};
use std::io;
use std::process::exit;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread::{self};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const DEFAULT_PORT: i32 = 9001;

type RoomKeys = Vec<UserKey>;
type LockedRoomKeys = Arc<Mutex<RoomKeys>>;

fn generate_key(passphrase_opt: Option<&str>) -> KeyData {
    let passphrase = match passphrase_opt {
        Some(p) => p.to_string(),
        None => String::from(""),
    };
    KeyData::from_passphrase(passphrase.as_bytes())
}

async fn refresh_roomkeys(
    stream: &mut TcpStream,
    roomkeys: &mut LockedRoomKeys,
) -> Result<(), Box<dyn std::error::Error>> {
    let message = APICommand::RefreshRoomKeysRequest;
    let serialized = serde_json::to_vec(&message)?;

    stream.write_all(&serialized).await?;

    let mut buffer = BytesMut::with_capacity(1024);
    stream.read_buf(&mut buffer).await?;

    let user_keys: Vec<UserKey> = serde_json::from_slice(&buffer)?;
    {
        let mut keymap = roomkeys.lock().unwrap();

        keymap.clear();
        for key in user_keys {
            keymap.push(key);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // We run a thing
    // let r: RoomKeys = Vec::new();

    let passphrase = "hello";
    let keys = generate_key(passphrase.into());

    let mut stream = match TcpStream::connect(format!("127.0.0.1:{}", DEFAULT_PORT)).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to connect to chat-server.");
            exit(1)
        }
    };

    let mut room_keys: LockedRoomKeys = Arc::new(Mutex::new(Vec::new()));
    if refresh_roomkeys(&mut stream, &mut room_keys).await.is_err() {
        eprintln!("Error Getting RoomKeys");
        exit(1);
    }

    let (reader, writer) = tokio::io::split(stream);

    /*
        Processes:
            1. Chatroom Listener
                Listens to messages send via the chatroom server. Decrypts pushes to stdout.
            2. Stdin Listener
                Listens to messages on StdIn. Encrypts messages with appropriate keys and sends them.
    */
    tokio::spawn(async move { chatroom_listener(reader, &keys).await });
    // tokio::spawn(async move { stdin_listener(writer) })
}

async fn chatroom_listener<T: AsyncReadExt + Unpin + Send>(
    mut socket_reader: T,
    my_keys: &KeyData,
) {
    let mut buffer: Vec<u8> = vec![0; 1024];
    loop {
        buffer.clear();

        let data = match socket_reader.read(&mut buffer).await {
            Ok(n) if n > 0 => &buffer[..n],
            _ => {
                eprintln!("Error Reading Socket. Disconnecting.");
                exit(1)
            }
        };

        let decrypted = decrypt(data.into(), &my_keys.private, &my_keys.passphrase);
        if let Ok(output) = str::from_utf8(&decrypted) {
            println!("{}", output)
        }
    }
}

async fn stdin_listener<T>(mut socket_writer: T, keys_mutex: LockedRoomKeys)
where
    T: AsyncWriteExt + Send + Unpin + 'static,
{
    /*  The StdIn Listener Process
     1. A blocking thread that listens to user input. The resulting user input
         is parsed, encrypted via the established chatserver keys, and
         sent to an async process which is responsible for writing the messages.
    */
    let (thread_writer, thread_reader) = std::sync::mpsc::channel();

    thread::spawn(move || loop {
        let mut buf = String::new();

        if io::stdin().read_line(&mut buf).is_err() {
            println!("Invalid Input");
            continue;
        };
        // TODO: Buf parser into message to send or command.
        {
            let keys = keys_mutex.lock().unwrap();

            let user_messages: Vec<Vec<u8>> = keys
                .iter()
                .map(|UserKey { user, public }| {
                    let message = encrypt(&buf, public);
                    serde_json::to_vec(&APICommand::Message(user.to_string(), message))
                        .expect("Failed to serialize message.")
                })
                .collect();

            for message in user_messages {
                thread_writer
                    .send(message)
                    .expect("Failure sending message");
            }
        };
    });

    tokio::spawn(async move {
        loop {
            let thread_message = thread_reader.recv().unwrap();

            if socket_writer.write_all(&thread_message).await.is_err() {
                eprintln!("Error writing message to socket: {:?}", thread_message);
                break;
            }
        }
    });
}
