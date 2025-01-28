pub mod feed;
pub mod hardwares;
pub mod nodes;
pub mod users;

use ntex::http::{Request, Response, StatusCode};
use ntex::web::Error;

use crate::constant::messages;
use crate::models::response::{ApiResponse, Data};
use crate::utils::http::serialize_response;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_not_found(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: messages::NOT_FOUND,
            data: Data::None,
        };
        let (data, status) = serialize_response(response, StatusCode::NOT_FOUND);
        Ok(response_json(data, status))
    }

    pub async fn handle_not_authenticated_with_message(
        &self,
        _: Request,
        message: &'static str,
    ) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message,
            data: Data::None,
        };
        let (data, status) = serialize_response(response, StatusCode::UNAUTHORIZED);
        Ok(response_json(data, status))
    }

    pub async fn handle_not_authorized(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: messages::UNAUTHORIZED,
            data: Data::None,
        };
        let (data, status) = serialize_response(response, StatusCode::FORBIDDEN);
        Ok(response_json(data, status))
    }

    pub async fn handle_bad_request(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: messages::INVALID_PAYLOAD,
            data: Data::None,
        };
        let (data, status) = serialize_response(response, StatusCode::BAD_REQUEST);
        Ok(response_json(data, status))
    }
}
