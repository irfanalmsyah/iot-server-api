use ntex::http::{Request, Response};
use ntex::web::Error;

use crate::database::nodes;
use crate::utils::auth::authenticate;
use crate::utils::http::extract_id_from_path;
use crate::{app::App, utils::http::response_json};

impl App {
    pub async fn handle_get_nodes(&self, req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(claims) => {
                let client = self.pool.get().await.unwrap();
                let (data, status) =
                    nodes::get_all_nodes(&client, claims.user_id, claims.isadmin).await;
                Ok(response_json(data, status))
            }
            Err(err) => self.handle_not_authenticated_with_message(req, err).await,
        }
    }

    pub async fn handle_get_node_by_id(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/nodes/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    let client = self.pool.get().await.unwrap();
                    let (data, status) =
                        nodes::get_node_with_feeds(&client, id, claims.user_id, claims.isadmin)
                            .await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_post_nodes(&self, mut req: Request) -> Result<Response, Error> {
        match authenticate(&req).await {
            Ok(claims) => {
                let payload = req.payload();
                let client = self.pool.get().await.unwrap();
                let (data, status) = nodes::add_node(&client, payload, claims.user_id).await;
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
                    let client = self.pool.get().await.unwrap();
                    let (data, status) =
                        nodes::update_node(&client, id, payload, claims.user_id, claims.isadmin)
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
                    let client = self.pool.get().await.unwrap();
                    let (data, status) =
                        nodes::delete_node(&client, id, claims.user_id, claims.isadmin).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }

    pub async fn handle_get_node_token(&self, req: Request) -> Result<Response, Error> {
        match extract_id_from_path(req.path(), "/token/") {
            Some(id) => match authenticate(&req).await {
                Ok(claims) => {
                    let client = self.pool.get().await.unwrap();
                    let (data, status) =
                        nodes::get_node_token(&client, id, claims.user_id, claims.isadmin).await;
                    Ok(response_json(data, status))
                }
                Err(err) => self.handle_not_authenticated_with_message(req, err).await,
            },
            None => self.handle_bad_request(req).await,
        }
    }
}
