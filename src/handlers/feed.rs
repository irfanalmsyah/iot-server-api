use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::utils::auth::authenticate;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_add_feed(&self, mut req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(claims) => {
                let payload = req.payload();
                let (data, status) = self.0.add_feed(payload, claims.user_id).await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }
}
