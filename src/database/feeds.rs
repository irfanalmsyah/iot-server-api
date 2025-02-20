use deadpool_postgres::Object;
use futures::StreamExt;

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};
use tokio_postgres::types::Type;

use crate::{
    constant::{messages, query},
    models::{
        feeds::{FeedPayload, MQTTFeedPayload},
        response::{ApiResponse, Data},
    },
    mqtt::ServerError,
    utils::http::serialize_response,
};

pub async fn add_feed(client: &Object, payload: &mut Payload, user_id: i32) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = std::str::from_utf8(&buf).unwrap();
    let data: FeedPayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<FeedPayload> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let stmt = client
        .prepare_typed_cached(query::NODES_SELECT_BY_ID, &[Type::INT4])
        .await
        .unwrap();
    let row = client.query_opt(&stmt, &[&data.node_id]).await.unwrap();
    if row.is_none() {
        let response: ApiResponse<FeedPayload> = ApiResponse {
            message: messages::NODE_NOT_FOUND,
            data: Data::None,
        };
        return serialize_response(response, StatusCode::NOT_FOUND);
    };
    if row.unwrap().get::<_, i32>(1) != user_id {
        let response: ApiResponse<FeedPayload> = ApiResponse {
            message: messages::UNAUTHORIZED,
            data: Data::None,
        };
        return serialize_response(response, StatusCode::UNAUTHORIZED);
    }

    let stmt = client
        .prepare_typed_cached(
            query::FEEDS_INSERT,
            &[Type::INT4, Type::TIMESTAMP, Type::FLOAT8_ARRAY],
        )
        .await
        .unwrap();

    match client
        .execute(
            &stmt,
            &[&data.node_id, &chrono::Utc::now().naive_utc(), &data.value],
        )
        .await
    {
        Ok(rows) => {
            if rows == 0 {
                let response: ApiResponse<FeedPayload> = ApiResponse {
                    message: messages::NODE_NOT_FOUND,
                    data: Data::None,
                };
                return serialize_response(response, StatusCode::NOT_FOUND);
            }
            let response: ApiResponse<FeedPayload> = ApiResponse {
                message: messages::CREATED,
                data: Data::None,
            };
            serialize_response(response, StatusCode::CREATED)
        }
        Err(e) => {
            let error_response: ApiResponse<FeedPayload> = ApiResponse {
                message: &e.to_string(),
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn add_feed_from_mqtt(
    client: &Object,
    data: MQTTFeedPayload,
    node_id: i32,
) -> Result<(), ServerError> {
    let stmt = client
        .prepare_typed_cached(
            query::FEEDS_INSERT,
            &[Type::INT4, Type::TIMESTAMP, Type::FLOAT8_ARRAY],
        )
        .await
        .unwrap();

    match client
        .execute(
            &stmt,
            &[&node_id, &chrono::Utc::now().naive_utc(), &data.value],
        )
        .await
    {
        Ok(rows) => {
            if rows == 0 {
                return Err(ServerError);
            }
        }
        Err(_) => return Err(ServerError),
    }
    Ok(())
}
