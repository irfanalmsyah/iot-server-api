use ntex::http::{Request, Response, StatusCode};
use ntex::web::Error;

use crate::utils::http::extract_id_from_path;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_nodes(&self, _: Request) -> Result<Response, Error> {
        let (data, status) = self.0.get_all_nodes().await;
        Ok(response_json(data, status))
    }

    pub async fn handle_get_node_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => {
                let (data, status) = self.0.get_node_with_feeds(id).await;
                Ok(response_json(data, status))
            }
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }
}
