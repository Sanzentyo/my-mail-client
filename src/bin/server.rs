use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

use my_mail_client::command::{SendCommand, Args};

const LOCAL: &str = "127.0.0.1:4747";
const MAX_BUFFER_SIZE: usize = 1024;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(LOCAL).await.unwrap();

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
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
                        },
                        Args::CheckMsg(_) => {
                            println!("CheckMsg received");
                        },
                    }

                    //tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    // jsonを送信
                    writer.write_all("Recieved".as_bytes()).await.unwrap();
                    writer.flush().await.unwrap();
                }
            }
        });
    }
}