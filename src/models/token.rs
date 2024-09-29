use std::{env, sync::LazyLock};

use chrono::Utc;
use dotenvy::dotenv;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::models::users::LoginInfoDTO;

pub static KEY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    dotenv().ok();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in the .env file");
    
    secret.into_bytes()
});

static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub iat: i64,
    pub exp: i64,
    pub user: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenBodyResponse {
    pub token: String
}

impl UserToken {
    pub fn generate_token(login: &LoginInfoDTO) -> String {
        dotenvy::dotenv().ok();
        let max_age: i64 = match env::var("MAX_AGE") {
            Ok(val) => val.parse::<i64>().unwrap_or(ONE_WEEK),
            Err(_) => ONE_WEEK,
        };

        let now = Utc::now().timestamp_nanos_opt().unwrap() / 1_000_000_000;
        let payload = UserToken {
            iat: now,
            exp: now + max_age,
            user: login.username.clone(),
        };

        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY[..]),
        )
        .unwrap()
    }
}
