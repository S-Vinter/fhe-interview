use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    KeyValue(KeyValue),
    GetRequest(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: Vec<u8>,
}
