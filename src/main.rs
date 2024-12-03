#[cfg(not(target_os = "macos"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use db::PgConnection;
use ntex::http::header::{CONTENT_TYPE, SERVER};
use ntex::http::{HttpService, KeepAlive::Os, Request, Response, StatusCode};
use ntex::server;
use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::web::{Error, HttpResponse};
use ntex::{time::Seconds, util::PoolId, util::Ready};
use std::io::Result as IoResult;
use std::sync::{Arc, Mutex};

mod db;
mod utils;

struct App(PgConnection);

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;

    async fn call(&self, req: Request, _: ServiceCtx<'_, Self>) -> Result<Response, Error> {
        match req.path() {
            "/db" => {
                let body = self.0.get_all_users().await;
                let mut res = HttpResponse::with_body(StatusCode::OK, body.into());
                res.headers_mut().insert(SERVER, utils::HDR_SERVER);
                res.headers_mut()
                    .insert(CONTENT_TYPE, utils::HDR_JSON_CONTENT_TYPE);
                Ok(res)
            }
            _ => Ok(Response::new(StatusCode::NOT_FOUND)),
        }
    }
}

struct AppFactory;

impl ServiceFactory<Request> for AppFactory {
    type Response = Response;
    type Error = Error;
    type Service = App;
    type InitError = ();

    async fn create(&self, _: ()) -> Result<Self::Service, Self::InitError> {
        const DB_URL: &str = "postgres://postgres:password@localhost/rustdemo";

        Ok(App(PgConnection::connect(DB_URL).await))
    }
}

#[ntex::main]
async fn main() -> IoResult<()> {
    println!("Starting http server: 127.0.0.1:8080");

    let cores = core_affinity::get_core_ids().unwrap();
    let total_cores = cores.len();
    let cores = Arc::new(Mutex::new(cores));

    server::build()
        .backlog(1024)
        .bind("techempower", "0.0.0.0:8080", |cfg| {
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(65535, 2048);
            PoolId::P1.set_write_params(65535, 2048);

            HttpService::build()
                .keep_alive(Os)
                .client_timeout(Seconds(0))
                .headers_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .payload_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .h1(AppFactory)
        })?
        .configure(move |cfg| {
            let cores = cores.clone();
            cfg.on_worker_start(move |_| {
                if let Some(core) = cores.lock().unwrap().pop() {
                    core_affinity::set_for_current(core);
                }
                Ready::<_, &str>::Ok(())
            });
            Ok(())
        })?
        .workers(total_cores)
        .run()
        .await
}
