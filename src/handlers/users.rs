use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::constant::messages;
use crate::database::users;
use crate::utils::auth::{authenticate, authenticate_admin};
use crate::utils::http::{extract_id_from_path, extract_jwt_from_path};
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_users(&self, req: Request) -> Result<Response, Error> {
        match authenticate_admin(&req).await {
            Ok(_) => {
                let client = self.pool.get().await.unwrap();
                let (data, status) = users::get_all_users(&client).await;
                Ok(response_json(data, status))
            }
            Err(err) if err == messages::UNAUTHORIZED => self.handle_not_authorized(req).await,
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_post_signup(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let client = self.pool.get().await.unwrap();
        let (data, status) = users::register_user(&client, payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_post_login(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let client = self.pool.get().await.unwrap();
        let (data, status) = users::login_user(&client, payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_get_user_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/users/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    if claims.user_id != id && !claims.isadmin {
                        return self.handle_not_authorized(req).await;
                    }
                    let client = self.pool.get().await.unwrap();
                    let (data, status) = users::get_one_user(&client, id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_activate_user(&self, req: Request) -> Result<Response, Error> {
        match extract_jwt_from_path(req.path(), "/activate/") {
            Some(token) => {
                let client = self.pool.get().await.unwrap();
                let (data, status) = users::activate_user(&client, token).await;
                Ok(response_json(data, status))
            }
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_forgot_password(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let client = self.pool.get().await.unwrap();
        let (data, status) = users::forgot_password(&client, payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_change_password(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let client = self.pool.get().await.unwrap();
        let (data, status) = users::change_password(&client, payload).await;
        Ok(response_json(data, status))
    }
}
