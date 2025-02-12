use deadpool_postgres::Object;
use futures::StreamExt;
use std::{borrow::Cow::Owned, str};
use tokio_postgres::types::Type;

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::{messages, query},
    models::{
        hardwares::{Hardware, HardwarePayload},
        response::{ApiResponse, Data},
    },
    utils::http::serialize_response,
};

pub async fn get_all_hardware(client: &Object) -> (Bytes, StatusCode) {
    let stmt = client
        .prepare_typed_cached(query::HARDWARES_SELECT, &[])
        .await
        .unwrap();
    let rows = client.query(&stmt, &[]).await.unwrap();

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
        message: messages::OK,
        data: Data::Multiple(hardwares),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn get_one_hardware(client: &Object, id: i32) -> (Bytes, StatusCode) {
    let stmt = client
        .prepare_typed_cached(query::HARDWARES_SELECT_BY_ID, &[Type::INT4])
        .await
        .unwrap();
    let rows = client.query(&stmt, &[&id]).await.unwrap();

    if rows.is_empty() {
        let error_response: ApiResponse<Hardware> = ApiResponse {
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

    let response = ApiResponse {
        message: messages::OK,
        data: Data::Single(hardware),
    };

    serialize_response(response, StatusCode::OK)
}

pub async fn add_hardware(client: &Object, payload: &mut Payload) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = std::str::from_utf8(&buf).unwrap();
    let data: HardwarePayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };
    if data.type_ != "sensor"
        && data.type_ != "single-board computer"
        && data.type_ != "microcontroller unit"
    {
        let error_response: ApiResponse<Hardware> = ApiResponse {
            message: messages::HARDWARE_TYPE_NOT_VALID,
            data: Data::None,
        };
        return serialize_response(error_response, StatusCode::BAD_REQUEST);
    }

    let stmt = client
        .prepare_typed_cached(
            query::HARDWARES_INSERT,
            &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR],
        )
        .await
        .unwrap();

    match client
        .execute(
            &stmt,
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
                message: messages::CREATED,
                data: Data::None,
            };
            serialize_response(response, StatusCode::CREATED)
        }
        Err(e) => {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: &e.to_string(),
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_hardware(
    client: &Object,
    id: i32,
    payload: &mut Payload,
) -> (Bytes, StatusCode) {
    let mut buf = Vec::new();
    while let Some(chunk) = payload.next().await {
        buf.extend_from_slice(&chunk.unwrap());
    }

    let data = std::str::from_utf8(&buf).unwrap();
    let data: HardwarePayload = match sonic_rs::from_str(data) {
        Ok(data) => data,
        Err(_) => {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: messages::INVALID_PAYLOAD,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }
    };

    let stmt = client
        .prepare_typed_cached(
            query::HARDWARES_UPDATE_BY_ID,
            &[Type::VARCHAR, Type::VARCHAR, Type::VARCHAR, Type::INT4],
        )
        .await
        .unwrap();

    match client
        .execute(
            &stmt,
            &[
                &data.name.as_ref(),
                &data.type_.as_ref(),
                &data.description.as_ref(),
                &id,
            ],
        )
        .await
    {
        Ok(rows_updated) => {
            if rows_updated == 0 {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: messages::HARDWARE_NOT_FOUND,
                    data: Data::None,
                };
                return serialize_response(error_response, StatusCode::NOT_FOUND);
            }
            let response: ApiResponse<HardwarePayload> = ApiResponse {
                message: messages::OK,
                data: Data::None,
            };
            serialize_response(response, StatusCode::OK)
        }
        Err(e) => {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: &e.to_string(),
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_hardware(client: &Object, id: i32) -> (Bytes, StatusCode) {
    let stmt = client
        .prepare_typed_cached(query::HARDWARES_DELETE_BY_ID, &[Type::INT4])
        .await
        .unwrap();
    match client.execute(&stmt, &[&id]).await {
        Ok(_) => {
            let response: ApiResponse<Hardware> = ApiResponse {
                message: messages::OK,
                data: Data::None,
            };
            serialize_response(response, StatusCode::OK)
        }
        Err(e) => {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: &e.to_string(),
                data: Data::None,
            };
            serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
