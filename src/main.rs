use dotenvy::dotenv;
use ntex::web;
use std::{env, io};

mod constants;
mod controllers;
mod db;
mod error;
mod models;
mod routes;
mod schema;
mod services;

#[ntex::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    db::init_pool(&database_url);

    let app = move || web::App::new().configure(routes::routes);

    web::server(app).bind("localhost:8080")?.run().await
}
