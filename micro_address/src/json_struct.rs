use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Data{
    pub nods: Vec<Node>
}

#[derive(Serialize, Deserialize)]
pub struct Node{
    pub id: String,
    pub address: Vec<String>
}

#[derive(Deserialize)]
pub struct WriteData{
    pub id: String,
    pub address: String
}
