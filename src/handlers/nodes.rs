use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::utils::auth::authenticate;
use crate::utils::http::extract_id_from_path;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_nodes(&self, req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(claims) => {
                let (data, status) = self.0.get_all_nodes(claims.user_id, claims.isadmin).await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_get_node_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    let (data, status) = self
                        .0
                        .get_node_with_feeds(id, claims.user_id, claims.isadmin)
                        .await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_post_nodes(&self, mut req: Request) -> Result<Response, Error> {
        let content_length = req
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        match authenticate(&req).await {
            Ok(claims) => {
                let payload = req.payload();
                let (data, status) = self
                    .0
                    .add_node(payload, claims.user_id, content_length)
                    .await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_update_node(&self, mut req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    let payload = req.payload();
                    let (data, status) = self
                        .0
                        .update_node(id, payload, claims.user_id, claims.isadmin)
                        .await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_delete_node(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    let (data, status) =
                        self.0.delete_node(id, claims.user_id, claims.isadmin).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }
}
