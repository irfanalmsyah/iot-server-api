use chrono::NaiveDateTime;
use sonic_rs::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Feed {
    pub node_id: i32,
    pub time: NaiveDateTime,
    pub value: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct FeedPayload {
    pub node_id: i32,
    pub value: Vec<f64>,
}
