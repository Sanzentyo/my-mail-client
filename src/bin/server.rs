use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;

use my_mail_client::{
    command::{SendCommand, Args, SendMsgResponse, ListMsgResponse, SearchMsgResponse, Message, ResponseStatus},
    db::{create_table, insert_msg, list_msg, search_msg},
};

use chrono::Utc;


const LOCAL: &str = "127.0.0.1:4747";
const MAX_BUFFER_SIZE: usize = 1024;
const DB_PATH: &str = "./msg.db";

#[tokio::main]
async fn main() {
    if !std::path::Path::new(DB_PATH).exists() {
        std::fs::File::create(DB_PATH).unwrap();
    }

    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}", DB_PATH)).await.unwrap();
    create_table(&pool).await.unwrap();
    let pool = Arc::new(pool);

    let listener = TcpListener::bind(LOCAL).await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let pool = pool.clone();
        println!("Accepted client: {:?}", addr);
        
        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            loop {
                // jsonを受信
                let mut buffer = [0; MAX_BUFFER_SIZE];
                while let Ok(n) = reader.read(&mut buffer).await {
                    if n == 0 {
                        println!("Client disconnected: {:?}", addr);
                        return;
                    }
                    let received = String::from_utf8_lossy(&buffer[..n]);

                    let recv_json = match serde_json::from_str::<SendCommand>(&received) {
                        Ok(input) => input,
                        Err(_) => {
                            writer.write_all("Invalid json".as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            continue;
                        },
                    };

                    match &recv_json.args {
                        Args::SendMsg(args) => {
                            println!("Message received: {:?}", args);
                            let timestamp = Utc::now().timestamp();
                            insert_msg(&pool, &recv_json.user_name, args, timestamp).await.unwrap();

                            let responce = SendMsgResponse {
                                status: ResponseStatus::Ok,
                                timestamp,
                            };

                            let json = serde_json::to_string(&responce).unwrap();
                            writer.write_all(json.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            println!("SendMsg sent");
                        },
                        Args::ListMsg(args) => {
                            let msgs = list_msg(&pool, args).await.unwrap();
                            
                            let response = ListMsgResponse {
                                status: ResponseStatus::Ok,
                                timestamp: Utc::now().timestamp(),
                                msg: msgs.into_iter().map(|msg| Message {
                                    from: msg.from_user,
                                    to: msg.to_user,
                                    content: msg.content,
                                    timestamp: msg.timestamp,
                                    uuid: msg.uuid,
                                    children_msg: Vec::new(), // TODO: implement recursive search
                                }).collect(),
                            };

                            let json = serde_json::to_string(&response).unwrap();
                            writer.write_all(json.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            println!("ListMsg sent");
                        },
                        Args::SearchMsg(args) => {
                            let msgs = search_msg(&pool, args, 0).await.unwrap();
                            
                            // Convert to tree structure
                            let mut msg_map = std::collections::HashMap::new();
                            let mut root_msgs: Vec<Message> = Vec::new();

                            // First, create a map of all messages
                            for msg in msgs {
                                let message = Message {
                                    from: msg.from_user,
                                    to: msg.to_user,
                                    content: msg.content,
                                    timestamp: msg.timestamp,
                                    uuid: msg.uuid,
                                    children_msg: Vec::new(),
                                };
                                msg_map.insert(msg.uuid, message);
                            }

                            // Then, build the tree structure
                            for msg in msg_map.values() {
                                if msg.uuid == args.select_uuid {
                                    root_msgs.push(msg.clone());
                                }
                            }

                            let response = SearchMsgResponse {
                                status: ResponseStatus::Ok,
                                timestamp: Utc::now().timestamp(),
                                msg: root_msgs,
                            };

                            let json = serde_json::to_string(&response).unwrap();
                            writer.write_all(json.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            println!("SearchMsg sent");
                        },
                    }

                    //tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        });
    }
}