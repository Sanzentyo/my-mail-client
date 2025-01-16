use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::io::{self, Write};

const SERVER_ADDR: &str = "127.0.0.1:4747";

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect(SERVER_ADDR).await?;
    let (reader, writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut writer = BufWriter::new(writer);
    
    loop {
        print!("Enter message (press 'Enter' to send): ");
        let mut input = String::new();
        io::stdout().flush()?;
        
        io::stdin().read_line(&mut input)?;
        
        writer.write_all(input.as_bytes()).await?;
        writer.flush().await?;

        println!("Message sent: {}", input.trim());
        
        let mut buffer = [0; 1024];
        let n = reader.read(&mut buffer).await?;
        println!("Response: {}", String::from_utf8_lossy(&buffer[..n]));
    }
}