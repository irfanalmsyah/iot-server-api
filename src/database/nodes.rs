use chrono::NaiveDateTime;
use futures::StreamExt;
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages::MESSAGE_OK,
    models::{
        feeds::Feed,
        nodes::{Node, NodePayload, NodeWithFeed},
        response::ApiResponse,
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
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

    /* pub async fn add_hardware(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
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
    } */

    pub async fn add_node(&self, payload: &mut Payload, user_id: i32) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<NodePayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.add_node,
                &[
                    &user_id,
                    &data.hardware_id,
                    &data.name.as_ref(),
                    &data.location.as_ref(),
                    &data.hardware_sensor_ids,
                    &data.hardware_sensor_names,
                    &data.ispublic,
                ],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<NodePayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![data],
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_node(&self, id: i32, payload: &mut Payload) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<NodePayload>(data).unwrap();

        match self
            .cl
            .execute(
                &self.update_node,
                &[
                    &data.hardware_id,
                    &data.name.as_ref(),
                    &data.location.as_ref(),
                    &data.hardware_sensor_ids,
                    &data.hardware_sensor_names,
                    &data.ispublic,
                    &id,
                ],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<NodePayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![data],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn delete_node(&self, id: i32) -> (Bytes, StatusCode) {
        match self.cl.execute(&self.delete_node, &[&id]).await {
            Ok(_) => {
                let response: ApiResponse<NodePayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
