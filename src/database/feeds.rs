use futures::StreamExt;

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages,
    models::{
        feeds::FeedPayload,
        response::{ApiResponse, Data},
    },
    mqtt::ServerError,
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn add_feed(&self, payload: &mut Payload, user_id: i32) -> (Bytes, StatusCode) {
        let mut buf = Vec::new();
        while let Some(chunk) = payload.next().await {
            buf.extend_from_slice(&chunk.unwrap());
        }

        let data = std::str::from_utf8(&buf).unwrap();
        let data = sonic_rs::from_str::<FeedPayload>(data).unwrap();
        match self
            .cl
            .execute(
                &self.feeds_insert,
                &[
                    &data.node_id,
                    &chrono::Utc::now().naive_utc(),
                    &data.value,
                    &user_id,
                ],
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
        &self,
        data: FeedPayload,
        user_id: i32,
    ) -> Result<(), ServerError> {
        match self
            .cl
            .execute(
                &self.feeds_insert,
                &[
                    &data.node_id,
                    &chrono::Utc::now().naive_utc(),
                    &data.value,
                    &user_id,
                ],
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
}
