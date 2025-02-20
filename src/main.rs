#[cfg(not(target_os = "macos"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use app::AppFactory;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use mqtt::{MySession, ServerError};
use ntex::http::{HttpService, KeepAlive::Os};
use ntex::service::fn_factory_with_config;
use ntex::{fn_service, server};
use ntex::{time::Seconds, util::PoolId, util::Ready};
use ntex_mqtt::{v3, v5};
use std::sync::{self, Arc};
use tokio_postgres::NoTls;

mod app;
mod constant;
mod database;
mod handlers;
mod models;
mod mqtt;
mod utils;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Starting http server: http://127.0.0.1:8080");
    println!("Starting mqtt server: mqtt://127.0.0.1:1883");

    let cores = core_affinity::get_core_ids().unwrap();
    let total_cores = cores.len();
    let cores = sync::Arc::new(sync::Mutex::new(cores));

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
    let pool_mqtt = pool.clone();

    let http_handle = server::build()
        .backlog(1024)
        .bind("serveriot", "0.0.0.0:8080", move |cfg| {
            let pool = pool.clone();
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(65535, 2048);
            PoolId::P1.set_write_params(65535, 2048);

            HttpService::build()
                .keep_alive(Os)
                .client_timeout(Seconds(0))
                .headers_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .payload_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .h1(AppFactory { pool: pool.clone() })
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
        .run();

    let mqtt_handle = server::build()
        .bind("mqtt", "0.0.0.0:1883", move |_| {
            let pg_connection_v3 = pool_mqtt.as_ref().clone();
            let pg_connection_v5 = pool_mqtt.as_ref().clone();
            ntex_mqtt::MqttServer::new()
                .v3(v3::MqttServer::new(mqtt::handle_handshake_v3)
                    .publish(fn_factory_with_config(
                        move |session: v3::Session<MySession>| {
                            let pg_connection = pg_connection_v3.clone();
                            Ready::Ok::<_, ServerError>(fn_service(move |req| {
                                let conn = pg_connection.clone();
                                mqtt::handle_publish_v3(session.clone(), req, conn)
                            }))
                        },
                    ))
                    .finish())
                .v5(v5::MqttServer::new(mqtt::handle_handshake_v5)
                    .publish(fn_factory_with_config(
                        move |session: v5::Session<MySession>| {
                            let pg_connection = pg_connection_v5.clone();
                            Ready::Ok::<_, ServerError>(fn_service(move |req| {
                                let conn = pg_connection.clone();
                                mqtt::handle_publish_v5(session.clone(), req, conn)
                            }))
                        },
                    ))
                    .finish())
        })?
        .run();

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install signal handler");
        println!("Shutdown signal received...");
    };

    let http_handle_clone = http_handle.clone();
    let mqtt_handle_clone = mqtt_handle.clone();

    tokio::select! {
        _ = http_handle => println!("HTTP server stopped."),
        _ = mqtt_handle => println!("MQTT server stopped."),
        _ = shutdown_signal => {
            println!("Stopping servers...");
            http_handle_clone.stop(true).await;
            mqtt_handle_clone.stop(true).await;
        }
    }

    println!("Servers shut down gracefully.");
    Ok(())
}
