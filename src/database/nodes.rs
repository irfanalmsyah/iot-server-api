use chrono::NaiveDateTime;
use futures::StreamExt;
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages,
    models::{
        feeds::Feed,
        hardwares::Hardware,
        nodes::{Node, NodePayload, NodeWithFeed},
        response::{ApiResponse, Data},
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn get_all_nodes(&self, user_id: i32, is_admin: bool) -> (Bytes, StatusCode) {
        let rows = if is_admin {
            self.cl.query(&self.nodes_select, &[]).await.unwrap()
        } else {
            self.cl
                .query(&self.nodes_select_by_user_and_ispublic, &[&user_id])
                .await
                .unwrap()
        };

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
            message: messages::OK,
            data: Data::Multiple(nodes),
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_node_with_feeds(
        &self,
        id: i32,
        user_id: i32,
        is_admin: bool,
    ) -> (Bytes, StatusCode) {
        let rows = if is_admin {
            self.cl
                .query(&self.nodes_select_by_id, &[&id])
                .await
                .unwrap()
        } else {
            self.cl
                .query(
                    &self.nodes_select_by_id_and_by_user_or_ispublic,
                    &[&id, &user_id],
                )
                .await
                .unwrap()
        };

        if rows.is_empty() {
            let error_response: ApiResponse<NodePayload> = ApiResponse {
                message: messages::NODE_NOT_FOUND,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::NOT_FOUND);
        }

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

        let feeds = self
            .cl
            .query(&self.feeds_select_by_node_id, &[&id])
            .await
            .unwrap();
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
            message: messages::OK,
            data: Data::Single(NodeWithFeed {
                node,
                feeds: feeds_data,
            }),
        };
        serialize_response(response, StatusCode::OK)
    }

    pub async fn add_node(
        &self,
        payload: &mut Payload,
        user_id: i32,
        content_length: usize,
    ) -> (Bytes, StatusCode) {
        let mut buf = Vec::with_capacity(content_length);
        while let Some(chunk) = payload.next().await {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(_) => {
                    let error_response: ApiResponse<NodePayload> = ApiResponse {
                        message: messages::INVALID_PAYLOAD,
                        data: Data::None,
                    };
                    return serialize_response(error_response, StatusCode::BAD_REQUEST);
                }
            };
            buf.extend_from_slice(&chunk);
        }

        let data: NodePayload = match unsafe { sonic_rs::from_slice_unchecked(&buf) } {
            Ok(data) => data,
            Err(_) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: messages::INVALID_PAYLOAD,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::BAD_REQUEST);
            }
        };

        let rows = self
            .cl
            .query(&self.hardwares_select_by_id, &[&data.hardware_id])
            .await
            .unwrap();
        if rows.is_empty() {
            let error_response: ApiResponse<NodePayload> = ApiResponse {
                message: messages::HARDWARE_NOT_FOUND,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::NOT_FOUND);
        }
        let hardware = Hardware {
            id: rows[0].get(0),
            name: Owned(rows[0].get::<_, &str>(1).to_string()),
            type_: Owned(rows[0].get::<_, &str>(2).to_string()),
            description: Owned(rows[0].get::<_, &str>(3).to_string()),
        };
        if hardware.type_ == "sensor" {
            let error_response: ApiResponse<NodePayload> = ApiResponse {
                message: messages::NODE_HARDWARE_CANNOT_BE_SENSOR,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }

        if data.hardware_sensor_ids.len() != data.hardware_sensor_names.len() {
            let error_response: ApiResponse<NodePayload> = ApiResponse {
                message: messages::SENSOR_ID_AND_SENSOR_NAME_MUST_HAVE_SAME_LENGTH,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }

        for id in &data.hardware_sensor_ids {
            let rows = self
                .cl
                .query(&self.hardwares_select_by_id, &[id])
                .await
                .unwrap();
            if rows.is_empty() {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: messages::SENSOR_NOT_FOUND,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::NOT_FOUND);
            }
            let hardware = Hardware {
                id: rows[0].get(0),
                name: Owned(rows[0].get::<_, &str>(1).to_string()),
                type_: Owned(rows[0].get::<_, &str>(2).to_string()),
                description: Owned(rows[0].get::<_, &str>(3).to_string()),
            };
            if hardware.type_ != "sensor" {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: messages::SENSOR_TYPE_NOT_VALID,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::BAD_REQUEST);
            }
        }

        match self
            .cl
            .execute(
                &self.nodes_insert,
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
                    message: messages::CREATED,
                    data: Data::None,
                };
                serialize_response(response, StatusCode::CREATED)
            }
            Err(e) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: &e.to_string(),
                    data: Data::None,
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_node(
        &self,
        id: i32,
        payload: &mut Payload,
        user_id: i32,
        is_admin: bool,
    ) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = str::from_utf8(&buf).unwrap();
        let data: NodePayload = match sonic_rs::from_str(data) {
            Ok(data) => data,
            Err(_) => {
                let error_response: ApiResponse<NodePayload> = ApiResponse {
                    message: messages::INVALID_PAYLOAD,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::BAD_REQUEST);
            }
        };
        if is_admin {
            match self
                .cl
                .execute(
                    &self.nodes_update_by_id,
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
                Ok(rows_updated) => {
                    if rows_updated == 0 {
                        let error_response: ApiResponse<NodePayload> = ApiResponse {
                            message: messages::NODE_NOT_FOUND,
                            data: Data::None,
                        };
                        return serialize_response(error_response, StatusCode::NOT_FOUND);
                    }
                    let response: ApiResponse<NodePayload> = ApiResponse {
                        message: messages::OK,
                        data: Data::None,
                    };
                    serialize_response(response, StatusCode::OK)
                }
                Err(e) => {
                    let error_response: ApiResponse<NodePayload> = ApiResponse {
                        message: &e.to_string(),
                        data: Data::None,
                    };
                    serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            match self
                .cl
                .execute(
                    &self.nodes_update_by_id_and_user_id,
                    &[
                        &data.hardware_id,
                        &data.name.as_ref(),
                        &data.location.as_ref(),
                        &data.hardware_sensor_ids,
                        &data.hardware_sensor_names,
                        &data.ispublic,
                        &id,
                        &user_id,
                    ],
                )
                .await
            {
                Ok(rows_updated) => {
                    if rows_updated == 0 {
                        let error_response: ApiResponse<NodePayload> = ApiResponse {
                            message: messages::NODE_NOT_FOUND,
                            data: Data::None,
                        };
                        return serialize_response(error_response, StatusCode::NOT_FOUND);
                    }
                    let response: ApiResponse<NodePayload> = ApiResponse {
                        message: messages::OK,
                        data: Data::Single(data),
                    };
                    serialize_response(response, StatusCode::OK)
                }
                Err(e) => {
                    let error_response: ApiResponse<NodePayload> = ApiResponse {
                        message: &e.to_string(),
                        data: Data::None,
                    };
                    serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }

    pub async fn delete_node(&self, id: i32, user_id: i32, is_admin: bool) -> (Bytes, StatusCode) {
        if is_admin {
            match self.cl.execute(&self.nodes_delete_by_id, &[&id]).await {
                Ok(_) => {
                    let response: ApiResponse<NodePayload> = ApiResponse {
                        message: messages::OK,
                        data: Data::None,
                    };
                    serialize_response(response, StatusCode::OK)
                }
                Err(e) => {
                    let error_response: ApiResponse<NodePayload> = ApiResponse {
                        message: &e.to_string(),
                        data: Data::None,
                    };
                    serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            match self
                .cl
                .execute(&self.nodes_delete_by_id_and_user_id, &[&id, &user_id])
                .await
            {
                Ok(_) => {
                    let response: ApiResponse<NodePayload> = ApiResponse {
                        message: messages::OK,
                        data: Data::None,
                    };
                    serialize_response(response, StatusCode::OK)
                }
                Err(e) => {
                    let error_response: ApiResponse<NodePayload> = ApiResponse {
                        message: &e.to_string(),
                        data: Data::None,
                    };
                    serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
    }
}
