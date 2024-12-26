use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::utils::auth::authenticate;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_users(&self, req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(_) => {
                let (data, status) = self.0.get_all_users().await;
                Ok(response_json(data, status))
            }
            Err(_) => self.handle_not_authenticated(req).await,
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
}
