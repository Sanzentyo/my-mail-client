use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use std::io::{self, Write};
use serde_json;

use my_mail_client::command::{
    read_json,
    SendCommand,
    Args, SendMsgArgs, CheckMsgArgs,
    SendMsgResponse, CheckMsgResponse, ResponseStatus
};

const SERVER_ADDR: &str = "127.0.0.1:4747";

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(SERVER_ADDR).await?;
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    
    loop {
        print!("Enter message (press 'Enter' to send): ");
        
        io::stdout().flush()?;
        
        let mut command_line = String::new();
        io::stdin().read_line(&mut command_line)?;

        let commands = command_line.split_whitespace().collect::<Vec<&str>>();

        match commands[0] {
            "check" => {
                if commands.len() < 2 {
                    eprintln!("Error: 'check' command requires a username argument.");
                    continue;
                }
                let from_user_name = commands[1].to_string();
                let command = SendCommand {
                    command: "check_msg".to_string(),
                    user_name: "user1".to_string(),
                    timestamp: 0,
                    args: Args::CheckMsg(CheckMsgArgs {
                        max_msg: -1,
                        recursive: -1,
                        from_user_name,
                        since: -1,
                        until: -1,
                    }),
                };
                let json = serde_json::to_string(&command).unwrap();
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                println!("Sent: {}", json);
                
                match read_json::<CheckMsgResponse>(&mut reader).await {
                    Ok(response) => {
                        match response.status {
                            ResponseStatus::Ok => {
                                println!("Received OK with timestamp: {}", response.timestamp);
                                // メッセージの処理
                                for msg in response.msg {
                                    println!("{:?}", msg);
                                }
                            },
                            ResponseStatus::Failed => {
                                println!("Received Failed with timestamp: {}", response.timestamp);
                            },
                            ResponseStatus::Invalid => {
                                println!("Received Invalid with timestamp: {}", response.timestamp);
                            },
                        }
                    },
                    Err(e) => {
                        println!("Failed to parse response: {:?}", e);
                    },
                }
            },
            _ => {
                let command = SendCommand {
                    command: "send_msg".to_string(),
                    user_name: "user1".to_string(),
                    timestamp: 0,
                    args: Args::SendMsg(SendMsgArgs {
                        to: "user2".to_string(),
                        content: command_line.trim().to_string(),
                        connected_id: -1,
                    }),
                };
                let json = serde_json::to_string(&command).unwrap();
                writer.write_all(json.as_bytes()).await?;
                writer.flush().await?;
                println!("Sent: {}", json);

                match read_json::<SendMsgResponse>(&mut reader).await {
                    Ok(response) => {
                        match response.status {
                            ResponseStatus::Ok => {
                                println!("Received OK with timestamp: {}", response.timestamp);
                            },
                            ResponseStatus::Failed => {
                                println!("Received Failed with timestamp: {}", response.timestamp);
                            },
                            ResponseStatus::Invalid => {
                                println!("Received Invalid with timestamp: {}", response.timestamp);
                            },
                        }
                    },
                    Err(e) => {
                        println!("Failed to parse response: {:?}", e);
                    },
                }
            }
        };
    }
}