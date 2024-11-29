use ntex::web;
use dotenvy::dotenv;
use std::{env, io};

mod routes;
mod controllers;
mod services;
mod schema;
mod models;
mod db;
mod error;
mod constants;

#[ntex::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = db::init_pool(&database_url).expect("Failed to create pool");
    
    let app = move || {
        web::App::new()
            .state(pool.clone())
            .configure(routes::routes)
    };

    web::server(app).bind("localhost:8080")?.run().await
}