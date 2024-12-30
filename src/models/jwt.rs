use std::borrow::Cow;

use sonic_rs::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub isadmin: bool,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ActivationClaims {
    pub username: Cow<'static, str>,
    pub exp: usize,
}
