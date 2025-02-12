use std::sync::Arc;

use deadpool_postgres::Pool;
use ntex::http::{Method, Request, Response};
use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::web::Error;

pub struct App {
    pub pool: Arc<Pool>,
}

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;

    async fn call(&self, req: Request, _: ServiceCtx<'_, Self>) -> Result<Response, Error> {
        match (req.path(), req.method()) {
            ("/users/", &Method::GET) => self.handle_get_users(req).await,
            ("/users/signup/", &Method::POST) => self.handle_post_signup(req).await,
            ("/users/login/", &Method::POST) => self.handle_post_login(req).await,
            ("/users/forgot-password/", &Method::POST) => self.handle_forgot_password(req).await,
            ("/users/change-password/", &Method::PUT) => self.handle_change_password(req).await,
            _ if req.path().starts_with("/users/") => match req.method() {
                &Method::GET => self.handle_get_user_by_id(req).await,
                _ => self.handle_not_found(req).await,
            },
            _ if req.path().starts_with("/activate/") => match req.method() {
                &Method::GET => self.handle_activate_user(req).await,
                _ => self.handle_not_found(req).await,
            },

            ("/hardwares/", &Method::GET) => self.handle_get_hardwares(req).await,
            ("/hardwares/", &Method::POST) => self.handle_post_hardwares(req).await,
            _ if req.path().starts_with("/hardwares/") => match req.method() {
                &Method::GET => self.handle_get_hardware_by_id(req).await,
                &Method::PUT => self.handle_update_hardware(req).await,
                &Method::DELETE => self.handle_delete_hardware(req).await,
                _ => self.handle_not_found(req).await,
            },

            ("/nodes/", &Method::GET) => self.handle_get_nodes(req).await,
            ("/nodes/", &Method::POST) => self.handle_post_nodes(req).await,
            _ if req.path().starts_with("/nodes/") => match req.method() {
                &Method::GET => self.handle_get_node_by_id(req).await,
                &Method::PUT => self.handle_update_node(req).await,
                &Method::DELETE => self.handle_delete_node(req).await,
                _ => self.handle_not_found(req).await,
            },

            ("/channel/", &Method::POST) => self.handle_add_feed(req).await,

            _ => self.handle_not_found(req).await,
        }
    }
}

pub struct AppFactory {
    pub pool: Arc<Pool>,
}

impl ServiceFactory<Request> for AppFactory {
    type Response = <App as Service<Request>>::Response;
    type Error = <App as Service<Request>>::Error;
    type Service = App;
    type InitError = ();

    async fn create(&self, _: ()) -> Result<Self::Service, Self::InitError> {
        Ok(App {
            pool: self.pool.clone(),
        })
    }
}
