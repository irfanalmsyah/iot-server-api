use futures::StreamExt;
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages::{self, MESSAGE_OK},
    models::{
        hardwares::{Hardware, HardwarePayload},
        response::{ApiResponse, Data},
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn get_all_hardware(&self) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.hardwares_select, &[]).await.unwrap();

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
            data: Data::Multiple(hardwares),
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_one_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        let rows = self
            .cl
            .query(&self.hardwares_select_by_id, &[&id])
            .await
            .unwrap();

        let hardware = Hardware {
            id: rows[0].get(0),
            name: Owned(rows[0].get::<_, &str>(1).to_string()),
            type_: Owned(rows[0].get::<_, &str>(2).to_string()),
            description: Owned(rows[0].get::<_, &str>(3).to_string()),
        };

        let response = ApiResponse {
            message: MESSAGE_OK,
            data: Data::Single(hardware),
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
        if data.name != "sensor"
            || data.name != "single-board computer"
            || data.name != "microcontroller unit"
        {
            let error_response: ApiResponse<Hardware> = ApiResponse {
                message: messages::HARDWARE_TYPE_NOT_VALID,
                data: Data::None,
            };
            return serialize_response(error_response, StatusCode::BAD_REQUEST);
        }

        match self
            .cl
            .execute(
                &self.hardwares_insert,
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
                    data: Data::Single(data),
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
                &self.hardwares_update_by_id,
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
                    data: Data::Single(data),
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

    pub async fn delete_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        match self.cl.execute(&self.hardwares_delete_by_id, &[&id]).await {
            Ok(_) => {
                let response: ApiResponse<Hardware> = ApiResponse {
                    message: MESSAGE_OK,
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
}
