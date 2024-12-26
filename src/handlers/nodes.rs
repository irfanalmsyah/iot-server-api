use ntex::http::{Request, Response, StatusCode};
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
                Ok(_) => {
                    let (data, status) = self.0.get_node_with_feeds(id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_post_nodes(&self, mut req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(claims) => {
                let payload = req.payload();
                let (data, status) = self.0.add_node(payload, claims.user_id).await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_update_node(&self, mut req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(_) => {
                    let payload = req.payload();
                    let (data, status) = self.0.update_node(id, payload).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }

    pub async fn handle_delete_node(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(_) => {
                    let (data, status) = self.0.delete_node(id).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => Ok(Response::new(StatusCode::BAD_REQUEST)),
        }
    }
}
