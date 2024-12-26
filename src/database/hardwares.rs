use futures::StreamExt;
use std::{borrow::Cow::Owned, str};

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages::MESSAGE_OK,
    models::{
        hardwares::{Hardware, HardwarePayload},
        response::ApiResponse,
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn get_all_hardware(&self) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.all_hardwares, &[]).await.unwrap();

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
            data: hardwares,
        };

        serialize_response(response, StatusCode::OK)
    }

    pub async fn get_one_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        let rows = self.cl.query(&self.one_hardware, &[&id]).await.unwrap();

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
            data: hardwares,
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
                &self.update_hardware,
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
                    data: vec![data],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn delete_hardware(&self, id: i32) -> (Bytes, StatusCode) {
        match self.cl.execute(&self.delete_hardware, &[&id]).await {
            Ok(_) => {
                let response: ApiResponse<Hardware> = ApiResponse {
                    message: MESSAGE_OK,
                    data: vec![],
                };
                serialize_response(response, StatusCode::OK)
            }
            Err(e) => {
                let error_response: ApiResponse<Hardware> = ApiResponse {
                    message: &e.to_string(),
                    data: vec![],
                };
                serialize_response(error_response, StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}