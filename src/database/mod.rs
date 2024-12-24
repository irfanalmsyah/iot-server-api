#![allow(clippy::uninit_vec)]
use std::str;

use tokio_postgres::{connect, Client, NoTls, Statement};

use crate::constant::query::{
    FEEDS_INSERT, FEEDS_SELECT_BY_NODE, HARDWARES_DELETE, HARDWARES_INSERT, HARDWARES_SELECT,
    HARDWARES_SELECT_ONE, HARDWARES_UPDATE, NODES_SELECT, NODES_SELECT_ONE, USERS_INSERT,
    USERS_LOGIN, USERS_SELECT,
};

pub mod feeds;
pub mod hardwares;
pub mod nodes;
pub mod users;

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
        let all_users = cl.prepare(USERS_SELECT).await.unwrap();
        let register_user = cl.prepare(USERS_INSERT).await.unwrap();
        let login_user = cl.prepare(USERS_LOGIN).await.unwrap();
        let all_hardwares = cl.prepare(HARDWARES_SELECT).await.unwrap();
        let one_hardware = cl.prepare(HARDWARES_SELECT_ONE).await.unwrap();
        let add_hardware = cl.prepare(HARDWARES_INSERT).await.unwrap();
        let update_hardware = cl.prepare(HARDWARES_UPDATE).await.unwrap();
        let delete_hardware = cl.prepare(HARDWARES_DELETE).await.unwrap();
        let all_nodes = cl.prepare(NODES_SELECT).await.unwrap();
        let one_node = cl.prepare(NODES_SELECT_ONE).await.unwrap();
        let feeds_by_node = cl.prepare(FEEDS_SELECT_BY_NODE).await.unwrap();
        let add_feed = cl.prepare(FEEDS_INSERT).await.unwrap();

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
