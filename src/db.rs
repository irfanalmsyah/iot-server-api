#![allow(clippy::uninit_vec)]
use std::{
    borrow::Cow::{self, Owned},
    cell::RefCell,
};

use ntex::{
    rt::spawn,
    util::{Bytes, BytesMut},
};
use sonic_rs::{to_writer, Serialize};
use tokio_postgres::{connect, Client, Statement};

use crate::utils::{reserve, BytesWriter};

#[derive(Serialize)]
pub struct User {
    pub id: i32,
    pub username: Cow<'static, str>,
    pub email: Cow<'static, str>,
    pub password: Cow<'static, str>,
    pub status: Option<bool>,
    pub isadmin: Option<bool>,
}

pub struct PgConnection {
    cl: Client,
    buf: RefCell<BytesMut>,
    all_users: Statement,
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> PgConnection {
        let (cl, conn) = connect(db_url)
            .await
            .expect("can not connect to postgresql");
        spawn(async move {
            let _ = conn.await;
        });
        let all_users = cl.prepare("SELECT * FROM users").await.unwrap();

        PgConnection {
            cl,
            buf: RefCell::new(BytesMut::with_capacity(10 * 1024 * 1024)),
            all_users,
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

        let mut body = self.buf.borrow_mut();
        reserve(&mut body, 10 * 1024 * 1024);
        to_writer(BytesWriter(&mut body), &users[..]).unwrap();
        body.split().freeze()
    }
}
