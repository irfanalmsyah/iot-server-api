#[cfg(not(target_os = "macos"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use app::AppFactory;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use ntex::http::{HttpService, KeepAlive::Os};
use ntex::server;
use ntex::{time::Seconds, util::PoolId, util::Ready};
use std::io::Result as IoResult;
use std::sync::{Arc, Mutex};
use tokio_postgres::NoTls;

mod app;
mod constant;
mod database;
mod handlers;
mod models;
mod utils;

#[ntex::main]
async fn main() -> IoResult<()> {
    println!("Starting http server: http://127.0.0.1:8080");

    let cores = core_affinity::get_core_ids().unwrap();
    let total_cores = cores.len();
    let cores = Arc::new(Mutex::new(cores));

    let mut cfg_pool = Config::new();
    cfg_pool.dbname = Some("rustdemo".to_string());
    cfg_pool.user = Some("postgres".to_string());
    cfg_pool.password = Some("password".to_string());
    cfg_pool.host = Some("localhost".to_string());
    cfg_pool.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg_pool.get_pool_config().max_size = 50;

    let pool: Pool = cfg_pool
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool");

    let pool = Arc::new(pool);

    server::build()
        .backlog(1024)
        .bind("techempower", "0.0.0.0:8080", {
            let pool = pool.clone();
            move |cfg| {
                cfg.memory_pool(PoolId::P1);
                PoolId::P1.set_read_params(65535, 2048);
                PoolId::P1.set_write_params(65535, 2048);

                HttpService::build()
                    .keep_alive(Os)
                    .client_timeout(Seconds(0))
                    .headers_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                    .payload_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                    .h1(AppFactory { pool: pool.clone() })
            }
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
