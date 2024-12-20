#![allow(clippy::uninit_vec)]
use futures::StreamExt;
use std::{
    borrow::Cow::{self, Owned},
    cell::RefCell,
};

use ntex::{
    http::Payload,
    util::{Bytes, BytesMut},
};
use serde::Deserialize;
use sonic_rs::{to_writer, Serialize};
use tokio_postgres::{connect, Client, NoTls, Statement};

use crate::{
    constant::MESSAGE_OK,
    utils::{reserve, BytesWriter},
};

#[derive(Serialize)]
struct ApiResponse<'a> {
    message: &'a str,
    data: Vec<User>,
}

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
pub struct RegisterPayload {
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,
}

pub struct PgConnection {
    cl: Client,
    buf: RefCell<BytesMut>,
    all_users: Statement,
    register_user: Statement,
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

        PgConnection {
            cl,
            buf: RefCell::new(BytesMut::with_capacity(10 * 1024 * 1024)),
            all_users,
            register_user,
        }
    }
}

impl PgConnection {
    pub async fn get_all_users(&self) -> Bytes {
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

        let mut body = self.buf.borrow_mut();
        reserve(&mut body, 10 * 1024 * 1024);
        to_writer(BytesWriter(&mut body), &response).unwrap();
        body.split().freeze()
    }

    pub async fn register_user(&self, payload: &mut Payload) -> Bytes {
        let mut body = self.buf.borrow_mut();
        reserve(&mut body, 10 * 1024 * 1024);

        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<RegisterPayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.register_user,
                &[
                    &data.username.as_ref(),
                    &data.email.as_ref(),
                    &data.password.as_ref(),
                ],
            )
            .await
        {
            Ok(_) => {
                to_writer(BytesWriter(&mut body), &data).unwrap();
                body.split().freeze()
            }
            Err(e) => {
                let error_response = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                to_writer(BytesWriter(&mut body), &error_response).unwrap();
                body.split().freeze()
            }
        }
    }
}
