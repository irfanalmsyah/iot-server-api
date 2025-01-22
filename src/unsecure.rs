use crate::{
    app, constant, database,
    mqtt::{self, MySession, ServerError},
};
use ntex::http;
use ntex::service::fn_factory_with_config;
use ntex::{fn_service, server};
use ntex::{time::Seconds, util::PoolId, util::Ready};
use ntex_mqtt::{v3, v5};
use std::sync;

pub async fn run_unsecure() -> std::io::Result<()> {
    println!("Starting http server: http://127.0.0.1:8080");
    println!("Starting mqtt server: mqtt://127.0.0.1:1883");

    let cores = core_affinity::get_core_ids().unwrap();
    let total_cores = cores.len();
    let cores = sync::Arc::new(sync::Mutex::new(cores));
    let pg_connection =
        sync::Arc::new(database::PgConnection::connect(constant::config::DB_URL).await);

    let http_handle = server::build()
        .backlog(1024)
        .bind("http", "0.0.0.0:8080", |cfg| {
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(65535, 2048);
            PoolId::P1.set_write_params(65535, 2048);

            http::HttpService::build()
                .keep_alive(http::KeepAlive::Os)
                .client_timeout(Seconds(0))
                .headers_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .payload_read_rate(Seconds::ZERO, Seconds::ZERO, 0)
                .h1(app::AppFactory)
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
            let pg_connection_v3 = pg_connection.clone();
            let pg_connection_v5 = pg_connection.clone();
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
