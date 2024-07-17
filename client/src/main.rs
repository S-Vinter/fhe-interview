use std::collections::HashMap;

use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::io::AsyncBufReadExt;
use rand::prelude::IteratorRandom;

fn xor_strings(s1: &str, s2: &str) -> Vec<u8> {
    // Convert strings to byte slices
    let bytes1 = s1.as_bytes();
    let bytes2 = s2.as_bytes();

    // Ensure both strings are of the same length
    let length = bytes1.len().min(bytes2.len());

    // Perform XOR operation
    (0..length)
        .map(|i| bytes1[i] ^ bytes2[i])
        .collect()
}

fn encrypt(secret: &str, msg: &str) -> Vec<u8> {
    xor_strings(secret, msg)
}

fn decrypt(secret: &str, msg: &str) -> Vec<u8> {
    xor_strings(secret, msg)
}

fn generate_random_string(length: usize) -> String {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| chars.chars().choose(&mut rng).unwrap())
        .collect()
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "127.0.0.1:8080";
    let mut stream = TcpStream::connect(addr).await?;

    let mut secrets = HashMap::<Vec<u8>, String>::new();

    println!("Connected to the server at {}", addr);

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let input = line.trim();

        if input.is_empty() {
            continue;
        }

        let secret = generate_random_string(input.len());
        let enc_value = encrypt(&secret, input);
        secrets.insert(enc_value.clone(), secret);

        // Send message to the server
        stream.write_all(&enc_value).await?;

        // Read the response
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await?;
        let input_from_server = String::from_utf8_lossy(&buf[..n]);
        println!("Received from server: {}", input_from_server);

        let secret = secrets.get(&input_from_server.to_string().as_bytes().to_vec()).unwrap();
        let decrypted = decrypt(secret, &input_from_server.to_string());
        unsafe {println!("{:?}", String::from_utf8_unchecked(decrypted));}

        if input == "exit" {
            println!("Disconnecting...");
            break;
        }
    }

    Ok(())
}
