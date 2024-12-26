use futures::StreamExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages::MESSAGE_OK,
    models::{
        jwt::Claims,
        response::{ApiResponse, Data},
        users::{LoginPayload, RegisterPayload, User, UserDTO},
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn get_all_users(&self) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.users_select, &[]).await.unwrap();

        let mut users = Vec::with_capacity(rows.len());
        for row in rows {
            users.push(User {
                id: row.get(0),
                username: Owned(row.get::<_, &str>(1).to_string()),
                email: Owned(row.get::<_, &str>(2).to_string()),
                password: Owned(row.get::<_, &str>(3).to_string()),
                status: row.get(4),
                isadmin: row.get(5),
            });
        }

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: Data::Multiple(users),
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn register_user(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<RegisterPayload>(data).unwrap();

        let hashed_password = bcrypt::hash(data.password.as_ref(), bcrypt::DEFAULT_COST).unwrap();

        match self
            .cl
            .execute(
                &self.users_insert,
                &[
                    &data.username.as_ref(),
                    &data.email.as_ref(),
                    &hashed_password,
                ],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<UserDTO> = ApiResponse {
                    message: MESSAGE_OK,
                    data: Data::Single(UserDTO {
                        username: data.username,
                        email: data.email,
                    }),
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<User> = ApiResponse {
                    message: &e.to_string(),
                    data: Data::None,
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn login_user(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<LoginPayload>(data).unwrap();

        let rows = self
            .cl
            .query(&self.users_select_by_username, &[&data.username.as_ref()])
            .await
            .unwrap();

        let mut users = Vec::with_capacity(rows.len());
        for row in rows {
            users.push(User {
                id: row.get(0),
                username: Owned(row.get::<_, &str>(1).to_string()),
                email: Owned(row.get::<_, &str>(2).to_string()),
                password: Owned(row.get::<_, &str>(3).to_string()),
                status: row.get(4),
                isadmin: row.get(5),
            });
        }

        if users.is_empty() || !bcrypt::verify(data.password.as_ref(), &users[0].password).unwrap()
        {
            let error_response: ApiResponse<User> = ApiResponse {
                message: "Invalid username or password",
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::UNAUTHORIZED);
        }

        let token = encode(
            &Header::default(),
            &Claims {
                user_id: users[0].id,
                isadmin: users[0].isadmin,
                exp: chrono::Utc::now().timestamp() as usize + 60 * 60,
            },
            &EncodingKey::from_secret("your-secret-key".as_ref()),
        )
        .unwrap();

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: Data::Single(token),
        };

        serialize_response(response, StatusCode::OK)
    }
}
