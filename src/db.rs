use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager, PooledConnection},
};

use std::sync::OnceLock;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
static DB_POOL: OnceLock<Pool> = OnceLock::new();

pub fn init_pool(url: &str) {
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = r2d2::Pool::builder()
        .min_idle(Some(4))
        .max_size(16)
        .build(manager)
        .expect("Failed to create pool.");

    DB_POOL.set(pool).expect("Pool was already set");
}

pub fn get_pool() -> &'static Pool {
    DB_POOL.get().expect("Pool is not initialized")
}

pub fn get_conn() -> PooledConnection<ConnectionManager<PgConnection>> {
    let conn = get_pool()
        .get()
        .expect("Failed to get a database connection");
    conn
}
