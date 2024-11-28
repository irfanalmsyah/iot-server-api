use sonic_rs::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseBody<T> {
    pub message: &'static str,
    pub data: T,
}

impl<T> ResponseBody<T> {
    pub fn new(message: &'static str, data: T) -> ResponseBody<T> {
        ResponseBody { message, data }
    }
}
