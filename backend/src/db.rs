use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn new_db_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set.");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(manager).unwrap()
}
