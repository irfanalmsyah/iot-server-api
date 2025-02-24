use chrono::{Duration, Utc};
use deadpool_postgres::Object;
use futures::StreamExt;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, SmtpTransport,
    Transport,
};
use std::{borrow::Cow::Owned, str};
use tokio_postgres::{error::SqlState, types::Type};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::{config, messages, query},
    models::{
        jwt::{ActivationClaims, Claims},
        response::{ApiResponse, Data},
        users::{
            ChangePasswordPayload, ForgotPasswordPayload, LoginPayload, RegisterPayload, User,
            UserDTO,
        },
    },
    utils::{generate_string, http::serialize_response},
};

pub async fn get_all_users(client: &Object) -> (Bytes, StatusCode) {
    let stmt = client
        .prepare_typed_cached(query::USERS_SELECT, &[])
        .await
        .unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();

    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        users.push(UserDTO {
            id: row.get(0),
            username: Owned(row.get::<_, &str>(1).to_string()),
            email: Owned(row.get::<_, &str>(2).to_string()),
            status: row.get(3),
            isadmin: row.get(4),
        });
    }

    let response = ApiResponse {
        message: messages::OK,
        data: Data::Multiple(users),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn register_user(client: &Object, payload: &mut Payload) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = std::str::from_utf8(&buf).unwrap();
    let data: RegisterPayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<User> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let hashed_password = bcrypt::hash(data.password.as_ref(), bcrypt::DEFAULT_COST).unwrap();

    let stmt = client
        .prepare_typed_cached(
            query::USERS_INSERT,
            &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR],
        )
        .await
        .unwrap();
    match client
        .execute(
            &stmt,
            &[
                &data.username.as_ref(),
                &data.email.as_ref(),
                &hashed_password,
            ],
        )
        .await
    {
        Ok(_) => {
            if config::ENVIROMENT == "development" {
                let response: ApiResponse<UserDTO> = ApiResponse {
                    message: messages::CREATED,
                    data: Data::None,
                };
                return serialize_response(response, StatusCode::CREATED);
            }
            let token = encode(
                &Header::default(),
                &ActivationClaims {
                    username: data.username,
                    exp: Utc::now()
                        .checked_add_signed(Duration::days(30))
                        .unwrap()
                        .timestamp() as usize,
                },
                &EncodingKey::from_secret(config::ACTIVATION_JWT_SECRET.as_ref()),
            );
            let body = format!(
                "Click this link to activate your account: http://localhost:8080/activate/{}/",
                token.unwrap()
            );
            let email = lettre::Message::builder()
                .from(config::EMAIL.parse().unwrap())
                .to(data.email.as_ref().parse().unwrap())
                .subject("Activate your account")
                .header(ContentType::TEXT_PLAIN)
                .body(body)
                .unwrap();

            let creds = Credentials::new(
                config::EMAIL_USERNAME.to_string(),
                config::EMAIL_PASSWORD.to_string(),
            );

            let mailer = SmtpTransport::relay(config::EMAIL_RELAY)
                .unwrap()
                .credentials(creds)
                .build();

            match mailer.send(&email) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            }

            let response: ApiResponse<UserDTO> = ApiResponse {
                message: messages::CREATED,
                data: Data::None,
            };
            serialize_response(response, StatusCode::CREATED)
        }
        Err(e) => {
            match e.code() {
                Some(code) => {
                    if *code == SqlState::UNIQUE_VIOLATION {
                        let error_response: ApiResponse<User> = ApiResponse {
                            message: messages::USER_EXISTS,
                            data: Data::None,
                        };
                        return serialize_response(error_response, StatusCode::CONFLICT);
                    }
                }
                None => (),
            }
            let error_response: ApiResponse<User> = ApiResponse {
                message: &e.to_string(),
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn login_user(client: &Object, payload: &mut Payload) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = std::str::from_utf8(&buf).unwrap();
    let data: LoginPayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<User> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let stmt = client
        .prepare_typed_cached(query::USERS_SELECT_BY_USERNAME, &[Type::VARCHAR])
        .await
        .unwrap();

    let rows = client
        .query(&stmt, &[&data.username.as_ref()])
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

    if users.is_empty() || !bcrypt::verify(data.password.as_ref(), &users[0].password).unwrap() {
        let error_response: ApiResponse<User> = ApiResponse {
            message: messages::LOGIN_FAILED,
            data: Data::None,
        };
        return serialize_response(error_response, StatusCode::UNAUTHORIZED);
    }
    if users[0].status == false {
        let error_response: ApiResponse<User> = ApiResponse {
            message: messages::ACCOUNT_NOT_ACTIVATED,
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
        &EncodingKey::from_secret(config::JWT_SECRET.as_ref()),
    )
    .unwrap();

    let response = ApiResponse {
        message: messages::OK,
        data: Data::Single(token),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn get_one_user(client: &Object, id: i32) -> (Bytes, StatusCode) {
    let stmt = client
        .prepare_typed_cached(query::USERS_SELECT_BY_ID, &[Type::INT4])
        .await
        .unwrap();
    let rows = client.query(&stmt, &[&id]).await.unwrap();

    if rows.is_empty() {
        let error_response: ApiResponse<User> = ApiResponse {
            message: messages::USER_NOT_FOUND,
            data: Data::None,
        };
        return serialize_response(error_response, StatusCode::NOT_FOUND);
    }

    let user = UserDTO {
        id: rows[0].get(0),
        username: Owned(rows[0].get::<_, &str>(1).to_string()),
        email: Owned(rows[0].get::<_, &str>(2).to_string()),
        status: rows[0].get(3),
        isadmin: rows[0].get(4),
    };

    let response = ApiResponse {
        message: messages::OK,
        data: Data::Single(user),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn activate_user(client: &Object, token: String) -> (Bytes, StatusCode) {
    match jsonwebtoken::decode::<ActivationClaims>(
        &token,
        &DecodingKey::from_secret(config::ACTIVATION_JWT_SECRET.as_ref()),
        &jsonwebtoken::Validation::default(),
    ) {
        Ok(token_data) => {
            let stmt = client
                .prepare_typed_cached(query::USERS_UPDATE_STATUS_BY_USERNAME, &[Type::VARCHAR])
                .await
                .unwrap();

            let rows = client
                .execute(&stmt, &[&token_data.claims.username])
                .await
                .unwrap();

            if rows == 0 {
                let error_response: ApiResponse<User> = ApiResponse {
                    message: messages::USER_NOT_FOUND,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::NOT_FOUND);
            }

            let response: ApiResponse<User> = ApiResponse {
                message: messages::OK,
                data: Data::None,
            };

            serialize_response(response, StatusCode::OK)
        }
        Err(_) => {
            let error_response: ApiResponse<User> = ApiResponse {
                message: messages::INVALID_TOKEN,
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn forgot_password(client: &Object, payload: &mut Payload) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = str::from_utf8(&buf).unwrap();
    let data: ForgotPasswordPayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<User> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let stmt = client
        .prepare_typed_cached(
            query::USERS_SELECT_BY_USERNAME_AND_EMAIL,
            &[Type::VARCHAR, Type::VARCHAR],
        )
        .await
        .unwrap();

    let rows = client
        .query(&stmt, &[&data.username.as_ref(), &data.email.as_ref()])
        .await
        .unwrap();

    if rows.is_empty() {
        let error_response: ApiResponse<User> = ApiResponse {
            message: messages::USER_NOT_FOUND,
            data: Data::None,
        };
        return serialize_response(error_response, StatusCode::NOT_FOUND);
    }

    let new_password = generate_string(16);
    let hashed_password = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST).unwrap();

    let stmt = client
        .prepare_typed_cached(
            query::USERS_UPDATE_PASSWORD_BY_USERNAME,
            &[Type::VARCHAR, Type::VARCHAR],
        )
        .await
        .unwrap();

    match client
        .execute(&stmt, &[&hashed_password, &data.username.as_ref()])
        .await
    {
        Ok(_) => {
            if config::ENVIROMENT == "development" {
                let response: ApiResponse<User> = ApiResponse {
                    message: messages::OK,
                    data: Data::None,
                };
                return serialize_response(response, StatusCode::OK);
            }
            let body = format!("Your new password is: {}", new_password);
            let email = lettre::Message::builder()
                .from(config::EMAIL.parse().unwrap())
                .to(data.email.as_ref().parse().unwrap())
                .subject("Forgot password")
                .header(ContentType::TEXT_PLAIN)
                .body(body)
                .unwrap();

            let creds = Credentials::new(
                config::EMAIL_USERNAME.to_string(),
                config::EMAIL_PASSWORD.to_string(),
            );

            let mailer = SmtpTransport::relay(config::EMAIL_RELAY)
                .unwrap()
                .credentials(creds)
                .build();

            match mailer.send(&email) {
                Ok(_) => (),
                Err(e) => eprintln!("{}", e),
            }

            let response: ApiResponse<User> = ApiResponse {
                message: messages::OK,
                data: Data::None,
            };
            serialize_response(response, StatusCode::OK)
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

pub async fn change_password(client: &Object, payload: &mut Payload) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = str::from_utf8(&buf).unwrap();
    let data: ChangePasswordPayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<User> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let stmt = client
        .prepare_typed_cached(query::USERS_SELECT_BY_USERNAME, &[Type::VARCHAR])
        .await
        .unwrap();

    let rows = client
        .query(&stmt, &[&data.username.as_ref()])
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

    if users.is_empty() || !bcrypt::verify(data.password.as_ref(), &users[0].password).unwrap() {
        let error_response: ApiResponse<User> = ApiResponse {
            message: messages::LOGIN_FAILED,
            data: Data::None,
        };
        return serialize_response(error_response, StatusCode::UNAUTHORIZED);
    }

    let hashed_password = bcrypt::hash(data.new_password.as_ref(), bcrypt::DEFAULT_COST).unwrap();

    let stmt = client
        .prepare_typed_cached(
            query::USERS_UPDATE_PASSWORD_BY_USERNAME,
            &[Type::VARCHAR, Type::VARCHAR],
        )
        .await
        .unwrap();

    match client
        .execute(&stmt, &[&hashed_password, &data.username.as_ref()])
        .await
    {
        Ok(_) => {
            let response: ApiResponse<User> = ApiResponse {
                message: messages::OK,
                data: Data::None,
            };
            serialize_response(response, StatusCode::OK)
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
