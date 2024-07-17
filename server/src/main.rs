use std::collections::HashMap;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};

pub struct DataBase {
    client_ip: String,
    data: HashMap<String, String>,
}

pub struct ServerDataBases {
    dbs: Vec<DataBase>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    let clients = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        let clients = Arc::clone(&clients);
        tokio::spawn(async move {
            handle_client(socket, addr, clients).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream, addr: std::net::SocketAddr, clients: Arc<Mutex<Vec<std::net::SocketAddr>>>) {
    {
        let mut clients = clients.lock().unwrap();
        clients.push(addr);
    }

    let mut buf = [0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Client disconnected: {}", addr);
                break;
            }
            Ok(n) => {
                let msg = String::from_utf8_lossy(&buf[..n]);
                println!("Received from {}: {}", addr, msg);
                // Echo message back to client
                if let Err(e) = socket.write_all(&buf[..n]).await {
                    println!("Failed to send to {}: {:?}", addr, e);
                    break;
                }
            }
            Err(e) => {
                println!("Error while reading from {}: {:?}", addr, e);
                break;
            }
        }
    }

    {
        let mut clients = clients.lock().unwrap();
        clients.retain(|&client_addr| client_addr != addr);
    }
}
