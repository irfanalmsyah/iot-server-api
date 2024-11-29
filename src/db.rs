use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

pub type PgPool = Pool<AsyncPgConnection>;

pub fn init_pool(url: &str) -> Result<Pool<AsyncPgConnection>, Box<dyn std::error::Error>> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
    let pool = Pool::builder(config).build()?;
    Ok(pool)
}
