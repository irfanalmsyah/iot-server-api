use ntex::http::{Request, Response, StatusCode};
use ntex::web::Error;

use crate::constant::messages::MESSAGE_UNAUTHORIZED;
use crate::utils::auth::{authenticate, authenticate_admin};
use crate::utils::http::{extract_id_from_path, extract_jwt_from_path};
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_users(&self, req: Request) -> Result<Response, Error> {
        match authenticate_admin(&req).await {
            Ok(_) => {
                let (data, status) = self.0.get_all_users().await;
                Ok(response_json(data, status))
            }
            Err(err) if err == MESSAGE_UNAUTHORIZED => self.handle_not_authorized(req).await,
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_post_signup(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let (data, status) = self.0.register_user(payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_post_login(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let (data, status) = self.0.login_user(payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_get_user_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/users/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    if claims.user_id != id && !claims.isadmin {
                        return self.handle_not_authorized(req).await;
                    }
                    let (data, status) = self.0.get_one_user(id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_activate_user(&self, req: Request) -> Result<Response, Error> {
        match extract_jwt_from_path(req.path(), "/activate/") {
            Some(token) => {
                let (data, status) = self.0.activate_user(token).await;
                Ok(response_json(data, status))
            }
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_forgot_password(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let (data, status) = self.0.forgot_password(payload).await;
        Ok(response_json(data, status))
    }

    pub async fn handle_change_password(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let (data, status) = self.0.change_password(payload).await;
        Ok(response_json(data, status))
    }
}
