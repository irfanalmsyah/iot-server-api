pub mod feed;
pub mod hardwares;
pub mod nodes;
pub mod users;

use ntex::http::{Request, Response, StatusCode};
use ntex::web::Error;

use crate::constant::messages::{MESSAGE_NOT_FOUND, MESSAGE_UNAUTHENTICATED, MESSAGE_UNAUTHORIZED};
use crate::models::response::ApiResponse;
use crate::utils::http::serialize_response;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_not_found(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: MESSAGE_NOT_FOUND,
            data: vec![],
        };
        let (data, status) = serialize_response(response, StatusCode::NOT_FOUND);
        Ok(response_json(data, status))
    }

    pub async fn handle_not_authenticated(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: MESSAGE_UNAUTHENTICATED,
            data: vec![],
        };
        let (data, status) = serialize_response(response, StatusCode::UNAUTHORIZED);
        Ok(response_json(data, status))
    }

    pub async fn handle_not_authorized(&self, _: Request) -> Result<Response, Error> {
        let response: ApiResponse<()> = ApiResponse {
            message: MESSAGE_UNAUTHORIZED,
            data: vec![],
        };
        let (data, status) = serialize_response(response, StatusCode::FORBIDDEN);
        Ok(response_json(data, status))
    }
}
