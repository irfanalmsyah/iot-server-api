#![allow(clippy::uninit_vec)]
use chrono::NaiveDateTime;
use futures::StreamExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};
use tokio_postgres::{connect, Client, NoTls, Statement};

use crate::{
    constant::MESSAGE_OK,
    models::{
        feeds::{Feed, FeedPayload},
        hardwares::{Hardware, HardwarePayload},
        jwt::Claims,
        nodes::{Node, NodeWithFeed},
        response::ApiResponse,
        users::{LoginPayload, RegisterPayload, User, UserDTO},
    },
    utils::{http::serialize_response, auth::authenticate},
};

pub struct PgConnection {
    cl: Client,
    all_users: Statement,
    register_user: Statement,
    login_user: Statement,
    all_hardwares: Statement,
    one_hardware: Statement,
    add_hardware: Statement,
    update_hardware: Statement,
    delete_hardware: Statement,
    all_nodes: Statement,
    one_node: Statement,
    feeds_by_node: Statement,
    add_feed: Statement,
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> PgConnection {
        let (cl, conn) = connect(db_url, NoTls)
            .await
            .expect("can not connect to postgresql");
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });
        let all_users = cl.prepare("SELECT * FROM users").await.unwrap();
        let register_user = cl
            .prepare("INSERT INTO users (username, email, password, status, isadmin) VALUES ($1, $2, $3, true, false)")
            .await
            .unwrap();
        let login_user = cl
            .prepare("SELECT * FROM users WHERE username = $1")
            .await
            .unwrap();
        let all_hardwares = cl.prepare("SELECT * FROM hardwares").await.unwrap();
        let one_hardware = cl
            .prepare("SELECT * FROM hardwares WHERE id = $1")
            .await
            .unwrap();
        let add_hardware = cl
            .prepare("INSERT INTO hardwares (name, type, description) VALUES ($1, $2, $3)")
            .await
            .unwrap();
        let update_hardware = cl
            .prepare("UPDATE hardwares SET name = $1, type = $2, description = $3 WHERE id = $4")
            .await
            .unwrap();
        let delete_hardware = cl
            .prepare("DELETE FROM hardwares WHERE id = $1")
            .await
            .unwrap();
        let all_nodes = cl.prepare("SELECT * FROM nodes").await.unwrap();
        let one_node = cl
            .prepare("SELECT * FROM nodes WHERE id = $1")
            .await
            .unwrap();
        let feeds_by_node = cl
            .prepare("SELECT * FROM feeds WHERE node_id = $1")
            .await
            .unwrap();
        let add_feed = cl
            .prepare("INSERT INTO feeds (node_id, time, value) VALUES ($1, $2, $3)")
            .await
            .unwrap();

        PgConnection {
            cl,
            all_users,
            register_user,
            login_user,
            all_hardwares,
            one_hardware,
            add_hardware,
            update_hardware,
            delete_hardware,
            all_nodes,
            one_node,
            feeds_by_node,
            add_feed,
        }
    }
}

impl PgConnection {
    pub async fn get_all_users(&self) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.all_users, &[]).await.unwrap();

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
            data: users,
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
                &self.register_user,
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
                    data: vec![UserDTO {
                        username: data.username,
                        email: data.email,
                    }],
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<User> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
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
            .query(&self.login_user, &[&data.username.as_ref()])
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
                data: vec![],
            };
            return serialize_response(error_response, StatusCode::UNAUTHORIZED);
        }

        let token = encode(
            &Header::default(),
            &Claims {
                user_id: users[0].id,
                exp: chrono::Utc::now().timestamp() as usize + 60 * 60,
            },
            &EncodingKey::from_secret("your-secret-key".as_ref()),
        )
        .unwrap();

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: vec![token],
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_all_hardware(&self, token: Option<&str>) -> (Bytes, StatusCode) {
        match authenticate(token).await {
            Ok(_) => (),
            Err(e) => return serialize_response(e, StatusCode::UNAUTHORIZED),
        }

        let rows = self.cl.query(&self.all_hardwares, &[]).await.unwrap();

        let mut hardwares = Vec::with_capacity(rows.len());
        for row in rows {
            hardwares.push(Hardware {
                id: row.get(0),
                name: Owned(row.get::<_, &str>(1).to_string()),
                type_: Owned(row.get::<_, &str>(2).to_string()),
                description: Owned(row.get::<_, &str>(3).to_string()),
            });
        }

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: hardwares,
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_one_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.one_hardware, &[&id]).await.unwrap();

        let mut hardwares = Vec::with_capacity(rows.len());
        for row in rows {
            hardwares.push(Hardware {
                id: row.get(0),
                name: Owned(row.get::<_, &str>(1).to_string()),
                type_: Owned(row.get::<_, &str>(2).to_string()),
                description: Owned(row.get::<_, &str>(3).to_string()),
            });
        }

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: hardwares,
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn add_hardware(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<HardwarePayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.add_hardware,
                &[
                    &data.name.as_ref(),
                    &data.type_.as_ref(),
                    &data.description.as_ref(),
                ],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<HardwarePayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![data],
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_hardware(&self, id: i32, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<HardwarePayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.update_hardware,
                &[
                    &data.name.as_ref(),
                    &data.type_.as_ref(),
                    &data.description.as_ref(),
                    &id,
                ],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<HardwarePayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![data],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn delete_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        match self.cl.execute(&self.delete_hardware, &[&id]).await {
            Ok(_) => {
                let response: ApiResponse<Hardware> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn get_all_nodes(&self) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.all_nodes, &[]).await.unwrap();

        let mut nodes = Vec::with_capacity(rows.len());
        for row in rows {
            nodes.push(Node {
                id: row.get(0),
                user_id: row.get(1),
                hardware_id: row.get(2),
                name: Owned(row.get::<_, &str>(3).to_string()),
                location: Owned(row.get::<_, &str>(4).to_string()),
                hardware_sensor_ids: row.get::<_, Vec<i32>>(5),
                hardware_sensor_names: row
                    .get::<_, Vec<&str>>(6)
                    .iter()
                    .map(|s| Owned(s.to_string()))
                    .collect(),
                ispublic: row.get(7),
            });
        }

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: nodes,
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_node_with_feeds(&self, id: i32) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.one_node, &[&id]).await.unwrap();

        let node = Node {
            id: rows[0].get(0),
            user_id: rows[0].get(1),
            hardware_id: rows[0].get(2),
            name: Owned(rows[0].get::<_, &str>(3).to_string()),
            location: Owned(rows[0].get::<_, &str>(4).to_string()),
            hardware_sensor_ids: rows[0].get::<_, Vec<i32>>(5),
            hardware_sensor_names: rows[0]
                .get::<_, Vec<&str>>(6)
                .iter()
                .map(|s| Owned(s.to_string()))
                .collect(),
            ispublic: rows[0].get(7),
        };

        let feeds = self.cl.query(&self.feeds_by_node, &[&id]).await.unwrap();
        let mut feeds_data = Vec::with_capacity(feeds.len());
        for row in feeds {
            feeds_data.push(Feed {
                id: row.get(0),
                node_id: row.get(1),
                time: row.get::<_, NaiveDateTime>(2),
                value: row.get::<_, Vec<f64>>(3),
            });
        }
        let response = ApiResponse {
            message: MESSAGE_OK,
            data: vec![NodeWithFeed {
                node,
                feeds: feeds_data,
            }],
        };
        serialize_response(response, StatusCode::OK)
    }

    pub async fn add_feed(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.add_feed,
                &[&data.node_id, &chrono::Utc::now().naive_utc(), &data.value],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<FeedPayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![data],
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<FeedPayload> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
