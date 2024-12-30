#![allow(clippy::uninit_vec)]
use std::str;

use tokio_postgres::{connect, Client, NoTls, Statement};

use crate::constant::query;

pub mod feeds;
pub mod hardwares;
pub mod nodes;
pub mod users;

pub struct PgConnection {
    cl: Client,
    users_select: Statement,
    users_insert: Statement,
    users_select_by_username: Statement,
    users_select_by_id: Statement,
    users_select_by_username_and_email: Statement,
    users_update_status_by_username: Statement,
    users_update_password_by_username: Statement,
    hardwares_select: Statement,
    hardwares_select_by_id: Statement,
    hardwares_insert: Statement,
    hardwares_update_by_id: Statement,
    hardwares_delete_by_id: Statement,
    nodes_select: Statement,
    nodes_select_by_user_and_ispublic: Statement,
    nodes_select_by_id: Statement,
    nodes_select_by_id_and_by_user_or_ispublic: Statement,
    nodes_insert: Statement,
    nodes_update_by_id: Statement,
    nodes_update_by_id_and_user_id: Statement,
    nodes_delete_by_id: Statement,
    nodes_delete_by_id_and_user_id: Statement,
    feeds_select_by_node_id: Statement,
    feeds_insert: Statement,
    hardwares_validate_sensor_ids: Statement,
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

        let users_select = cl.prepare(query::USERS_SELECT).await.unwrap();
        let users_insert = cl.prepare(query::USERS_INSERT).await.unwrap();
        let users_select_by_username = cl.prepare(query::USERS_SELECT_BY_USERNAME).await.unwrap();
        let users_select_by_id = cl.prepare(query::USERS_SELECT_BY_ID).await.unwrap();
        let users_select_by_username_and_email = cl
            .prepare(query::USERS_SELECT_BY_USERNAME_AND_EMAIL)
            .await
            .unwrap();
        let users_update_status_by_username = cl
            .prepare(query::USERS_UPDATE_STATUS_BY_USERNAME)
            .await
            .unwrap();
        let users_update_password_by_username = cl
            .prepare(query::USERS_UPDATE_PASSWORD_BY_USERNAME)
            .await
            .unwrap();
        let hardwares_select = cl.prepare(query::HARDWARES_SELECT).await.unwrap();
        let hardwares_select_by_id = cl.prepare(query::HARDWARES_SELECT_BY_ID).await.unwrap();
        let hardwares_insert = cl.prepare(query::HARDWARES_INSERT).await.unwrap();
        let hardwares_update_by_id = cl.prepare(query::HARDWARES_UPDATE_BY_ID).await.unwrap();
        let hardwares_delete_by_id = cl.prepare(query::HARDWARES_DELETE_BY_ID).await.unwrap();
        let nodes_select = cl.prepare(query::NODES_SELECT).await.unwrap();
        let nodes_select_by_user_and_ispublic = cl
            .prepare(query::NODES_SELECT_BY_USER_OR_ISPUBLIC)
            .await
            .unwrap();
        let nodes_select_by_id = cl.prepare(query::NODES_SELECT_BY_ID).await.unwrap();
        let nodes_select_by_id_and_by_user_or_ispublic = cl
            .prepare(query::NODES_SELECT_BY_ID_AND_BY_USER_OR_ISPUBLIC)
            .await
            .unwrap();
        let nodes_insert = cl.prepare(query::NODES_INSERT).await.unwrap();
        let nodes_update_by_id = cl.prepare(query::NODES_UPDATE_BY_ID).await.unwrap();
        let nodes_update_by_id_and_user_id = cl
            .prepare(query::NODES_UPDATE_BY_ID_AND_USER_ID)
            .await
            .unwrap();
        let nodes_delete_by_id = cl.prepare(query::NODES_DELETE_BY_ID).await.unwrap();
        let nodes_delete_by_id_and_user_id = cl
            .prepare(query::NODES_DELETE_BY_ID_AND_USER_ID)
            .await
            .unwrap();
        let feeds_select_by_node_id = cl.prepare(query::FEEDS_SELECT_BY_NODE_ID).await.unwrap();
        let feeds_insert = cl.prepare(query::FEEDS_INSERT).await.unwrap();
        let hardwares_validate_sensor_ids = cl
            .prepare(query::HARDWARES_VALIDATE_SENSOR_IDS)
            .await
            .unwrap();

        PgConnection {
            cl,
            users_select,
            users_insert,
            users_select_by_username,
            users_select_by_id,
            users_select_by_username_and_email,
            users_update_status_by_username,
            users_update_password_by_username,
            hardwares_select,
            hardwares_select_by_id,
            hardwares_insert,
            hardwares_update_by_id,
            hardwares_delete_by_id,
            nodes_select,
            nodes_select_by_user_and_ispublic,
            nodes_select_by_id,
            nodes_select_by_id_and_by_user_or_ispublic,
            nodes_insert,
            nodes_update_by_id,
            nodes_update_by_id_and_user_id,
            nodes_delete_by_id,
            nodes_delete_by_id_and_user_id,
            feeds_select_by_node_id,
            feeds_insert,
            hardwares_validate_sensor_ids,
        }
    }
}
