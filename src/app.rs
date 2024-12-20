use ntex::http::header::{CONTENT_TYPE, SERVER};
use ntex::http::{Request, Response, StatusCode};
use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::web::Error;
use ntex_bytes::Bytes;
use crate::db::PgConnection;
use crate::utils;

pub struct App(pub PgConnection);

fn response_success(data: Bytes) -> Response {
    let mut res = Response::with_body(StatusCode::OK, data.into());
    res.headers_mut()
        .insert(CONTENT_TYPE, utils::HDR_JSON_CONTENT_TYPE);
    res.headers_mut().insert(SERVER, utils::HDR_SERVER);
    res
}

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;

    async fn call(&self, mut req: Request, _: ServiceCtx<'_, Self>) -> Result<Response, Error> {
        match (req.path(), req.method()) {
            ("/users/", &ntex::http::Method::GET) => {
                let body = self.0.get_all_users().await;
                Ok(response_success(body))
            }
            ("/users/signup/", &ntex::http::Method::POST) => {
                let payload = req.payload();
                let body = self.0.register_user(payload).await;
                Ok(response_success(body))
            }
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
