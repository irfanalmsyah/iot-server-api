use crate::database::PgConnection;
use ntex::http::{Method, Request, Response, StatusCode};
use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::web::Error;

pub struct App(pub PgConnection);

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;

    async fn call(&self, req: Request, _: ServiceCtx<'_, Self>) -> Result<Response, Error> {
        match (req.path(), req.method()) {
            ("/users/", &Method::GET) => self.handle_get_users().await,
            ("/users/signup/", &Method::POST) => self.handle_post_signup(req).await,
            ("/users/login/", &Method::POST) => self.handle_post_login(req).await,

            ("/hardwares/", &Method::GET) => self.handle_get_hardwares(req).await,
            ("/hardwares/", &Method::POST) => self.handle_post_hardwares(req).await,
            _ if req.path().starts_with("/hardwares/") && req.method() == &Method::GET => {
                self.handle_get_hardware_by_id(req).await
            }
            _ if req.path().starts_with("/hardwares/") && req.method() == &Method::PUT => {
                self.handle_update_hardware(req).await
            }
            _ if req.path().starts_with("/hardwares/") && req.method() == &Method::DELETE => {
                self.handle_delete_hardware(req).await
            }

            ("/nodes/", &Method::GET) => self.handle_get_nodes().await,
            _ if req.path().starts_with("/nodes/") && req.method() == &Method::GET => {
                self.handle_get_node_by_id(req).await
            }

            ("/channel/", &Method::POST) => self.handle_add_feed(req).await,

            _ => Ok(Response::new(StatusCode::NOT_FOUND)),
        }
    }
}

pub struct AppFactory;

impl ServiceFactory<Request> for AppFactory {
    type Response = <App as Service<Request>>::Response;
    type Error = <App as Service<Request>>::Error;
    type Service = App;
    type InitError = ();

    async fn create(&self, _: ()) -> Result<Self::Service, Self::InitError> {
        const DB_URL: &str = "postgres://postgres:password@localhost/rustdemo";

        Ok(App(PgConnection::connect(DB_URL).await))
    }
}
