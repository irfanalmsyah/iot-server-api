use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::constant::messages;
use crate::database::hardwares;
use crate::utils::auth::{authenticate, authenticate_admin};
use crate::utils::http::extract_id_from_path;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_hardwares(&self, req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(_) => {
                let client = self.pool.get().await.unwrap();
                let (data, status) = hardwares::get_all_hardware(&client).await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_post_hardwares(&self, mut req: Request) -> Result<Response, Error> {
        match authenticate_admin(&req).await {
            Ok(_) => {
                let payload = req.payload();
                let client = self.pool.get().await.unwrap();
                let (data, status) = hardwares::add_hardware(&client, payload).await;
                Ok(response_json(data, status))
            }
            Err(err) if err == messages::UNAUTHORIZED => self.handle_not_authorized(req).await,
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_get_hardware_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate(&req).await {
                Ok(_) => {
                    let client = self.pool.get().await.unwrap();
                    let (data, status) = hardwares::get_one_hardware(&client, id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_update_hardware(&self, mut req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate_admin(&req).await {
                Ok(_) => {
                    let payload = req.payload();
                    let client = self.pool.get().await.unwrap();
                    let (data, status) = hardwares::update_hardware(&client, id, payload).await;
                    Ok(response_json(data, status))
                }
                Err(err) if err == messages::UNAUTHORIZED => self.handle_not_authorized(req).await,
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_delete_hardware(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/hardwares/") {
            Some(id) => match authenticate_admin(&req).await {
                Ok(_) => {
                    let client = self.pool.get().await.unwrap();
                    let (data, status) = hardwares::delete_hardware(&client, id).await;
                    Ok(response_json(data, status))
                }
                Err(err) if err == messages::UNAUTHORIZED => self.handle_not_authorized(req).await,
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }
}
