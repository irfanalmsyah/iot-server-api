use ntex::http::{Request, Response, StatusCode};
use ntex::web::Error;

use crate::constant::messages::MESSAGE_UNAUTHORIZED;
use crate::utils::auth::{authenticate, authenticate_admin};
use crate::utils::http::extract_id_from_path;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_hardwares(&self, req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(_) => {
                let (data, status) = self.0.get_all_hardware().await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_post_hardwares(&self, mut req: Request) -> Result<Response, Error> {
        match authenticate_admin(&req).await {
            Ok(_) => {
                let payload = req.payload();
                let (data, status) = self.0.add_hardware(payload).await;
                Ok(response_json(data, status))
            }
            Err(err) if err == MESSAGE_UNAUTHORIZED => self.handle_not_authorized(req).await,
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_get_hardware_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate(&req).await {
                Ok(_) => {
                    let (data, status) = self.0.get_one_hardware(id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_update_hardware(&self, mut req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate_admin(&req).await {
                Ok(_) => {
                    let payload = req.payload();
                    let (data, status) = self.0.update_hardware(id, payload).await;
                    Ok(response_json(data, status))
                }
                Err(err) if err == MESSAGE_UNAUTHORIZED => self.handle_not_authorized(req).await,
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_delete_hardware(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate_admin(&req).await {
                Ok(_) => {
                    let (data, status) = self.0.delete_hardware(id).await;
                    Ok(response_json(data, status))
                }
                Err(err) if err == MESSAGE_UNAUTHORIZED => self.handle_not_authorized(req).await,
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }
}
