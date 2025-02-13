use chrono::NaiveDateTime;
use deadpool_postgres::Object;
use futures::StreamExt;
use std::{borrow::Cow::Owned, collections::HashMap, str};
use tokio_postgres::types::Type;

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::{messages, query},
    models::{
        feeds::Feed,
        hardwares::Hardware,
        nodes::{Node, NodePayload, NodeWithFeed},
        response::{ApiResponse, Data},
    },
    utils::http::serialize_response,
};

pub async fn get_all_nodes(client: &Object, user_id: i32, is_admin: bool) -> (Bytes, StatusCode) {
    let rows = if is_admin {
        let stmt = client
            .prepare_typed_cached(query::NODES_SELECT, &[])
            .await
            .unwrap();
        client.query(&stmt, &[]).await.unwrap()
    } else {
        let stmt = client
            .prepare_typed_cached(query::NODES_SELECT_BY_USER_OR_ISPUBLIC, &[Type::INT4])
            .await
            .unwrap();
        client.query(&stmt, &[&user_id]).await.unwrap()
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
    let id_nodes: Vec<i32> = nodes.iter().map(|node| node.id).collect();
    let stmt = client
        .prepare_typed_cached(query::FEEDS_SELECT_BY_NODE_IDS, &[Type::INT4_ARRAY])
        .await
        .unwrap();
    let feed_rows = client.query(&stmt, &[&id_nodes]).await.unwrap();
    let mut feeds_by_node: HashMap<i32, Vec<Feed>> = HashMap::new();
    for row in feed_rows {
        let feed = Feed {
            time: row.get::<_, NaiveDateTime>(0),
            value: row.get::<_, Vec<f64>>(1),
            node_id: row.get(2),
        };
        feeds_by_node.entry(feed.node_id).or_default().push(feed);
    }

    let mut node_with_feed = Vec::with_capacity(nodes.len());
    for node in nodes {
        let node_feeds = feeds_by_node.remove(&node.id).unwrap_or_default();
        node_with_feed.push(NodeWithFeed {
            node,
            feeds: node_feeds,
        });
    }

    let response = ApiResponse {
        message: messages::OK,
        data: Data::Multiple(node_with_feed),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn get_node_with_feeds(
    client: &Object,
    id: i32,
    user_id: i32,
    is_admin: bool,
) -> (Bytes, StatusCode) {
    let rows = if is_admin {
        let stmt = client
            .prepare_typed_cached(query::NODES_SELECT_BY_ID, &[Type::INT4])
            .await
            .unwrap();
        client.query(&stmt, &[&id]).await.unwrap()
    } else {
        let stmt = client
            .prepare_typed_cached(
                query::NODES_SELECT_BY_ID_AND_BY_USER_OR_ISPUBLIC,
                &[Type::INT4, Type::INT4],
            )
            .await
            .unwrap();
        client.query(&stmt, &[&id, &user_id]).await.unwrap()
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

    let stmt = client
        .prepare_typed_cached(query::FEEDS_SELECT_BY_NODE_ID, &[Type::INT4])
        .await
        .unwrap();

    let feeds = client.query(&stmt, &[&id]).await.unwrap();
    let mut feeds_data = Vec::with_capacity(feeds.len());
    for row in feeds {
        feeds_data.push(Feed {
            node_id: row.get(0),
            time: row.get::<_, NaiveDateTime>(1),
            value: row.get::<_, Vec<f64>>(2),
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

pub async fn add_node(client: &Object, payload: &mut Payload, user_id: i32) -> (Bytes, StatusCode) {
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

    let stmt = client
        .prepare_typed_cached(query::HARDWARES_SELECT_BY_ID, &[Type::INT4])
        .await
        .unwrap();

    let rows = client.query(&stmt, &[&data.hardware_id]).await.unwrap();
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
        let rows = client.query(&stmt, &[id]).await.unwrap();
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

    let stmt = client
        .prepare_typed_cached(
            query::NODES_INSERT,
            &[
                Type::INT4,
                Type::INT4,
                Type::TEXT,
                Type::TEXT,
                Type::INT4_ARRAY,
                Type::TEXT_ARRAY,
                Type::BOOL,
            ],
        )
        .await
        .unwrap();

    match client
        .execute(
            &stmt,
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
    client: &Object,
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
        let stmt = client
            .prepare_typed_cached(
                query::NODES_UPDATE_BY_ID,
                &[
                    Type::INT4,
                    Type::TEXT,
                    Type::TEXT,
                    Type::INT4_ARRAY,
                    Type::TEXT_ARRAY,
                    Type::BOOL,
                    Type::INT4,
                ],
            )
            .await
            .unwrap();
        match client
            .execute(
                &stmt,
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
        let stmt = client
            .prepare_typed_cached(
                query::NODES_UPDATE_BY_ID_AND_USER_ID,
                &[
                    Type::INT4,
                    Type::TEXT,
                    Type::TEXT,
                    Type::INT4_ARRAY,
                    Type::TEXT_ARRAY,
                    Type::BOOL,
                    Type::INT4,
                    Type::INT4,
                ],
            )
            .await
            .unwrap();

        match client
            .execute(
                &stmt,
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

pub async fn delete_node(
    client: &Object,
    id: i32,
    user_id: i32,
    is_admin: bool,
) -> (Bytes, StatusCode) {
    if is_admin {
        let stmt = client
            .prepare_typed_cached(query::NODES_DELETE_BY_ID, &[Type::INT4])
            .await
            .unwrap();
        match client.execute(&stmt, &[&id]).await {
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
        let stmt = client
            .prepare_typed_cached(
                query::NODES_DELETE_BY_ID_AND_USER_ID,
                &[Type::INT4, Type::INT4],
            )
            .await
            .unwrap();
        match client.execute(&stmt, &[&id, &user_id]).await {
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
