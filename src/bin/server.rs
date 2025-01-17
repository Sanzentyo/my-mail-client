use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::sync::Arc;

use rust_thread_messenger::{
    command::{SendCommand, Args, SendMsgResponse, ListMsgResponse, SearchMsgResponse, Message, ResponseStatus, InvaildResponse},
    db::{create_table, insert_msg, list_msg, search_msg},
};

use chrono::Utc;


const LOCAL: &str = "127.0.0.1:4747";
const MAX_BUFFER_SIZE: usize = 1<<16;
const DB_PATH: &str = "./msg.db";

#[tokio::main]
async fn main() {

    // パスにDBファイルが存在しない場合は作成
    if !std::path::Path::new(DB_PATH).exists() {
        std::fs::File::create(DB_PATH).unwrap();
    }

    // DB接続
    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}", DB_PATH)).await.unwrap();
    create_table(&pool).await.unwrap();
    let pool = Arc::new(pool); // スレッド間で共有するためArcでラップ

    // ポートをバインド
    let listener = TcpListener::bind(LOCAL).await.unwrap();

    // クライアントからの接続を待ち受けループを開始
    loop {
        // クライアントからの接続を待ち受け
        let (mut socket, addr) = listener.accept().await.unwrap();
        let pool = pool.clone();
        println!("Accepted client: {:?}", addr);
        
        // クライアントごとにスレッドを立てる
        tokio::spawn(async move {
            // クライアントとの通信を行うためのreader, writerに分割
            let (reader, writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            // 切断されるまでクライアントからのリクエストを待ち受け
            loop {
                // jsonを受信
                let mut buffer = Box::new([0; MAX_BUFFER_SIZE]); // Boxでヒープに確保
                while let Ok(n) = reader.read(&mut buffer[..]).await {
                    // クライアントが切断した場合
                    if n == 0 {
                        println!("Client disconnected: {:?}", addr);
                        return;
                    }
                    let received = String::from_utf8_lossy(&buffer[..n]);

                    // jsonをパース
                    let recv_json = match serde_json::from_str::<SendCommand>(&received) {
                        Ok(input) => input,
                        Err(_) => {
                            // パースに失敗した場合
                            let invaild_json = InvaildResponse {
                                status: ResponseStatus::Invalid,
                                timestamp: Utc::now().timestamp(),
                            };
                            let json = serde_json::to_string(&invaild_json).unwrap();

                            writer.write_all(json.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            continue;
                        },
                    };


                    // コマンドによって処理を分岐
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
                                    connected_id: msg.connected_msg_uuid,
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
                            let mut msg_map = std::collections::HashMap::new();
                            for m in msgs {
                                let message = Message {
                                    from: m.from_user,
                                    to: m.to_user,
                                    content: m.content,
                                    timestamp: m.timestamp,
                                    uuid: m.uuid,
                                    children_msg: Vec::new(),
                                    connected_id: m.connected_msg_uuid,
                                };
                                msg_map.insert(message.uuid, message);
                            }

                            // 子を設定
                            let mut updates = Vec::new();
                            for parent in msg_map.values() {
                                let p_uuid = parent.uuid;
                                let mut children = Vec::new();
                                for child in msg_map.values() {
                                    if child.connected_id == p_uuid {
                                        children.push(child.clone());
                                    }
                                }
                                updates.push((p_uuid, children));
                            }
                            for (uuid, children) in updates {
                                if let Some(p) = msg_map.get_mut(&uuid) {
                                    p.children_msg = children;
                                }
                            }

                            // ルート(connected_id = -1 か、存在しない)を探す
                            let mut root_msgs = Vec::new();
                            for m in msg_map.values() {
                                if m.connected_id == -1 || !msg_map.contains_key(&m.connected_id) {
                                    root_msgs.push(m.clone());
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