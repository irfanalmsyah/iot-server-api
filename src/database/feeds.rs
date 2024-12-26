use futures::StreamExt;

use ntex::{
    http::{Payload, StatusCode},
    util::Bytes,
};

use crate::{
    constant::messages::MESSAGE_OK,
    models::{
        feeds::FeedPayload,
        response::{ApiResponse, Data},
    },
    utils::http::serialize_response,
};

use super::PgConnection;

impl PgConnection {
    pub async fn add_feed(&self, payload: &mut Payload) -> (Bytes, StatusCode) {
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
                &[&data.node_id, &chrono::Utc::now().naive_utc(), &data.value],
            )
            .await
        {
            Ok(_) => {
                let response: ApiResponse<FeedPayload> = ApiResponse {
                    message: MESSAGE_OK,
                    data: Data::Single(data),
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
}
