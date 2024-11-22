use chrono::Utc;
use jsonwebtoken::{decode, encode, errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::{env, sync::LazyLock};

use crate::models::users::User;
include!(concat!(env!("OUT_DIR"), "/jwt_secret.rs"));

pub static ENCODING_KEY: LazyLock<EncodingKey> =
    LazyLock::new(|| EncodingKey::from_secret(JWT_SECRET));

pub static DECODING_KEY: LazyLock<DecodingKey> =
    LazyLock::new(|| DecodingKey::from_secret(JWT_SECRET));

pub static HEADER: LazyLock<Header> = LazyLock::new(|| Header::default());

pub static VALIDATION: LazyLock<Validation> = LazyLock::new(|| Validation::default());

static ONE_WEEK: i64 = 60 * 60 * 24 * 7;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserToken {
    pub iat: i64,
    pub exp: i64,
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub isadmin: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TokenBodyResponse {
    pub token: String,
}

impl UserToken {
    pub fn generate_token(user: &User) -> String {
        let now = Utc::now().timestamp();
        let claims = UserToken {
            iat: now,
            exp: now + ONE_WEEK,
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            isadmin: user.isadmin.unwrap_or(false),
        };

        encode(&HEADER, &claims, &ENCODING_KEY).unwrap()
    }

    pub fn verify_token(token: &str) -> Result<Self, Error> {
        decode::<Self>(token, &DECODING_KEY, &VALIDATION).map(|data| data.claims)
    }
}
