use sonic_rs::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub isadmin: bool,
    pub exp: usize,
}
