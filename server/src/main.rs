use std::collections::HashMap;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt};
use std::sync::{Arc, Mutex};

use common::{KeyValue, Request};

#[derive(Default)]
pub struct DataBase {
    data: HashMap<String, HashMap<String, Vec<u8>>>,
}

impl DataBase {
    pub fn new_client(&mut self, ip: &str) {
        self.data.insert(ip.to_string(), HashMap::new());
    }

    pub fn new_enc_data(&mut self, ip: &str, new_data: KeyValue) {
        match self.data.get_mut(ip) {
            Some(value_hashmap) => {
                value_hashmap.insert(new_data.key, new_data.value);
            },
            None => {
                println!("No such ip!");
            },
        }
    }

    pub fn get(&mut self, ip: &str, key: &str) -> Vec<u8> {
        match self.data.get_mut(ip) {
            Some(value_hashmap) => {
                match value_hashmap.get_mut(key) {
                    Some(enc_value) => {
                        return enc_value.to_vec();
                    },
                    None => {
                        println!("No such key!");
                        return Vec::new();
                    }
                }
            },
            None => {
                println!("No such ip!");
                return Vec::new();
            },
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");
    let db = Arc::new(Mutex::new(DataBase::default()));

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

                let request = bincode::deserialize::<Request>(msg.to_string().as_str().as_bytes()).unwrap();
                match request {
                    Request::GetRequest(key) => {
                        let mut db = db.lock().unwrap();
                        println!("{:?}", db.get(&format!("{addr}"), &key));
                    }
                    Request::KeyValue(key_value) => {
                        println!("{:?}", key_value);
                        let mut db = db.lock().unwrap();
                        db.new_enc_data(&format!("{}", addr), key_value);
                    }
                }

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
