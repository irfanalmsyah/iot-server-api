use diesel::{
    pg::PgConnection,
    r2d2::{self, ConnectionManager},
};

pub type Connection = PgConnection;

pub type Pool = r2d2::Pool<ConnectionManager<Connection>>;

pub fn init_pool(url: &str) -> Pool {
    let manager = ConnectionManager::<Connection>::new(url);
    let pool = r2d2::Pool::builder()
        .min_idle(Some(4))
        .max_size(16)
        .build(manager)
        .expect("Failed to create pool.");
    
    pool
}
