use diesel::pg::PgConnection;
use diesel::r2d2::{Builder, ConnectionManager, Pool};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn new_db_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set.");
    let max_db_connections = env::var("MAX_DB_CONNECTIONS")
        .expect("MAX_DB_CONNECTIONS not set.")
        .parse()
        .unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Builder::new().max_size(max_db_connections).build_unchecked(manager)
}
