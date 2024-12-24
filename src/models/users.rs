use std::borrow::Cow;

use sonic_rs::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,
    pub status: bool,
    pub isadmin: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserDTO {
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterPayload {
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginPayload {
    pub username: Cow<'static, str>,
    pub password: Cow<'static, str>,
}
