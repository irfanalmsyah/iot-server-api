use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_add_feed(&self, mut req: Request) -> Result<Response, Error> {
        let payload = req.payload();
        let (data, status) = self.0.add_feed(payload).await;
        Ok(response_json(data, status))
    }
}