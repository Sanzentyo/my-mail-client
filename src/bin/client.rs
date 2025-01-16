use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::io::{self, Write};
use serde_json;

use my_mail_client::command::{SendCommand, Args, SendMsgArgs, CheckMsgArgs};

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
        
        let mut content = String::new();
        io::stdin().read_line(&mut content)?;

        let input = match content.trim() {
            "check" => SendCommand {
                command: "check_msg".to_string(),
                user_name: "user1".to_string(),
                timestamp: 0,
                args: Args::CheckMsg(CheckMsgArgs {
                    max_msg: -1,
                    recursive: -1,
                    from_user_name: "".to_string(),
                    since: -1,
                    until: -1,
                }),
            },
            _ => SendCommand {
                command: "send_msg".to_string(),
                user_name: "user1".to_string(),
                timestamp: 0,
                args: Args::SendMsg(SendMsgArgs {
                    to: "user2".to_string(),
                    content: content.trim().to_string(),
                    connected_id: -1,
                }),
            }
            
        };
        
        let json = serde_json::to_string(&input)?;
        writer.write_all(json.as_bytes()).await?;
        writer.flush().await?;

        println!("Message sent: {:?}", input);
        
        let mut buffer = [0; 1024];
        let n = reader.read(&mut buffer).await?;
        println!("Response: {}", String::from_utf8_lossy(&buffer[..n]));
    }
}