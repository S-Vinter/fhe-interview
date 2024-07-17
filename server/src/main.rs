use std::collections::HashMap;
use std::sync::mpsc::channel;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc, Mutex};

use common::KeyValue;

#[derive(Default)]
pub struct DataBase {
    data: HashMap<String, HashMap<String, Vec<u8>>>,
}

impl DataBase {
    pub fn new_client(&mut self, ip: &str) {
        self.data.insert(ip.to_string(), HashMap::new());
    }

    pub fn new_enc_data(&mut self, ip: &str, new_data: KeyValue) {
        println!("{:?}", ip);
        match self.data.get_mut(ip) {
            Some((value_hashmap)) => {
                value_hashmap.insert(new_data.key, new_data.value);
            },
            None => {
                println!("No such ip!");
            },
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");
    let mut db = Arc::new(Mutex::new(DataBase::default()));

    let clients = Arc::new(Mutex::new(Vec::new()));

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New client connected: {}", addr);

        let clients = Arc::clone(&clients);
        let db = Arc::clone(&db);
        tokio::spawn(async move {
            handle_client(socket, addr, clients, db).await;
        });
    }
}

async fn handle_client(mut socket: TcpStream, addr: std::net::SocketAddr, clients: Arc<Mutex<Vec<std::net::SocketAddr>>>, db: Arc<Mutex<DataBase>>) {
    {
        let mut clients = clients.lock().unwrap();
        clients.push(addr);
        let mut db = db.lock().unwrap();
        db.new_client(&format!("{addr}"));
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

                let key_value = bincode::deserialize::<KeyValue>(msg.to_string().as_str().as_bytes()).unwrap();
                println!("{:?}", key_value);
                let mut db = db.lock().unwrap();
                db.new_enc_data(&format!("{}", addr), key_value);

                // Echo message back to client
                // if let Err(e) = socket.write_all(&buf[..n]).await {
                //     println!("Failed to send to {}: {:?}", addr, e);
                //     break;
                // }
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
