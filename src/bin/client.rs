use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use std::io::{self, Write};
use serde_json;

use my_mail_client::command::{
    read_json,
    SendCommand,
    Args, SendMsgArgs, ListMsgArgs, SearchMsgArgs,
    SendMsgResponse, ListMsgResponse, SearchMsgResponse, ResponseStatus,
    Message,
};

const SERVER_ADDR: &str = "127.0.0.1:4747";

// メインループでユーザ入力を繰り返し受け付ける
#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(SERVER_ADDR).await?;
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);

    println!("Connected to server: {}", SERVER_ADDR);

    println!("--------------------------------");
    print!("ユーザー名を入力してください...\n");
    print!("> ");
    io::stdout().flush()?;
    let mut user_name = String::new();
    std::io::stdin().read_line(&mut user_name)?;
    let user_name = user_name.trim().to_string();

    loop {
        println!("--------------------------------");
        println!("コマンドを入力してください (send, list, search)...");
        print!("> ");
        io::stdout().flush()?;

        let mut command_str = String::new();
        std::io::stdin().read_line(&mut command_str)?;
        let command_str = command_str.trim();

        match command_str {
            "send" => {
                // 引数を標準入力で取得
                let mut to = String::new();
                print!("宛先 (to) を入力してください: ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut to)?;
                let to = to.trim().to_string();

                let mut content = String::new();
                print!("メッセージ内容 (content) を入力してください: ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut content)?;
                let content = content.trim().to_string();

                let mut conn_id_input = String::new();
                print!("connected_id (なければ空白): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut conn_id_input)?;
                let connected_id: i64 = conn_id_input.trim().parse().unwrap_or(-1);

                let command = SendCommand {
                    command: "send_msg".to_string(),
                    user_name: user_name.clone(),
                    timestamp: 0,
                    args: Args::SendMsg(SendMsgArgs {
                        to,
                        content,
                        connected_id,
                    }),
                };
                let json = serde_json::to_string(&command).unwrap();
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                println!("Sent: {}", json);

                // レスポンスを受信
                match read_json::<SendMsgResponse>(&mut reader).await {
                    Ok(response) => match response.status {
                        ResponseStatus::Ok => {
                            println!("Received OK with timestamp: {}", response.timestamp);
                        }
                        ResponseStatus::Failed => {
                            println!("Received Failed with timestamp: {}", response.timestamp);
                        }
                        ResponseStatus::Invalid => {
                            println!("Received Invalid with timestamp: {}", response.timestamp);
                        }
                    },
                    Err(e) => println!("Failed to parse response: {:?}", e),
                }
            },
            "list" => {
                // 引数を標準入力で取得
                let mut max_msg_input = String::new();
                print!("max_msg (なければ空白): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut max_msg_input)?;
                let max_msg: i64 = max_msg_input.trim().parse().unwrap_or(-1);

                let mut recursive_input = String::new();
                print!("recursive (例: 0): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut recursive_input)?;
                let recursive: i64 = recursive_input.trim().parse().unwrap_or(0);

                let mut from_name = String::new();
                print!("from_user_name (なければ空文字): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut from_name)?;
                let from_user_name = from_name.trim().to_string();

                let mut to_name = String::new();
                print!("to_user_name (なければ空文字): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut to_name)?;
                let to_user_name = to_name.trim().to_string();

                let mut since_input = String::new();
                print!("since (なければ空白): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut since_input)?;
                let since: i64 = since_input.trim().parse().unwrap_or(-1);

                let mut until_input = String::new();
                print!("until (なければ空白): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut until_input)?;
                let until: i64 = until_input.trim().parse().unwrap_or(-1);

                let command = SendCommand {
                    command: "list_msg".to_string(),
                    user_name: user_name.clone(),
                    timestamp: 0,
                    args: Args::ListMsg(ListMsgArgs {
                        max_msg,
                        recursive,
                        from_user_name,
                        to_user_name,
                        since,
                        until,
                    }),
                };
                let json = serde_json::to_string(&command).unwrap();
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                println!("Sent: {}", json);

                // レスポンスを受信
                match read_json::<ListMsgResponse>(&mut reader).await {
                    Ok(response) => match response.status {
                        ResponseStatus::Ok => {
                            println!("Received OK with timestamp: {}", response.timestamp);
                            for msg in response.msg {
                                print_message(&msg, 0);
                            }
                        }
                        ResponseStatus::Failed => println!("Received Failed"),
                        ResponseStatus::Invalid => println!("Received Invalid"),
                    },
                    Err(e) => println!("Error: {:?}", e),
                }
            },
            "search" => {
                // 引数を標準入力で取得
                let mut uuid_input = String::new();
                print!("select_uuid: ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut uuid_input)?;
                let select_uuid: i64 = uuid_input.trim().parse().unwrap_or(-1);

                let mut recursive_input = String::new();
                print!("recursive (例: 0、しないなら空白): ");
                io::stdout().flush()?;
                std::io::stdin().read_line(&mut recursive_input)?;
                let recursive: i64 = recursive_input.trim().parse().unwrap_or(0);

                let command = SendCommand {
                    command: "search_msg".to_string(),
                    user_name: user_name.clone(),
                    timestamp: 0,
                    args: Args::SearchMsg(SearchMsgArgs {
                        select_uuid,
                        recursive,
                    }),
                };
                let json = serde_json::to_string(&command).unwrap();
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                println!("Sent: {}", json);

                // レスポンスを受信
                match read_json::<SearchMsgResponse>(&mut reader).await {
                    Ok(response) => match response.status {
                        ResponseStatus::Ok => {
                            println!("Received OK with timestamp: {}", response.timestamp);
                            for msg in response.msg {
                                print_message(&msg, 0);
                            }
                        }
                        ResponseStatus::Failed => println!("Received Failed"),
                        ResponseStatus::Invalid => println!("Received Invalid"),
                    },
                    Err(e) => println!("Error: {:?}", e),
                }
            },
            "" => {
                println!("空の入力です。再度入力してください。");
                continue;
            },
            _ => {
                println!("不明なコマンドです。");
                continue;
            },
        }
    }
}

fn print_message(msg: &Message, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}--------------------------------", indent);
    println!("{}From: {}", indent, msg.from);
    println!("{}To: {}", indent, msg.to);
    println!("{}Content: {}", indent, msg.content);
    println!("{}UUID: {}", indent, msg.uuid);
    println!("{}Timestamp: {}", indent, msg.timestamp);
    
    if !msg.children_msg.is_empty() {
        println!("{}Children:", indent);
        for child in &msg.children_msg {
            print_message(child, depth + 1);
        }
    }
}