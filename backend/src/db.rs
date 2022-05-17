use std::env;

use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::methods::LoadQuery;
use diesel::r2d2::{Builder, ConnectionManager, Pool};
use diesel::sql_types::BigInt;
use serde::Serialize;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn new_db_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set.");
    let max_db_connections = env::var("MAX_DB_CONNECTIONS")
        .expect("MAX_DB_CONNECTIONS not set.")
        .parse()
        .unwrap();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Builder::new()
        .max_size(max_db_connections)
        .build_unchecked(manager)
}

// Pagination shamelessly ripped from diesel example code

#[derive(Serialize)]
pub struct Page<T>
where
    T: Serialize,
{
    results: Vec<T>,
    page_count: i64,
    has_more: bool,
}

impl<T> Page<T>
where
    T: Serialize,
{
    pub fn new(results: Vec<T>, page_count: i64, has_more: bool) -> Self {
        Self {
            results,
            page_count,
            has_more,
        }
    }
}

pub trait Paginate: Sized {
    fn paginate(self, page_number: i64, per_page: i64) -> Paginated<Self>;
}

impl<T> Paginate for T {
    fn paginate(self, page_number: i64, per_page: i64) -> Paginated<Self> {
        Paginated {
            query: self,
            per_page,
            page_number,
            offset: page_number * per_page,
        }
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page_number: i64,
    per_page: i64,
    offset: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated {
            per_page,
            offset: self.page_number * per_page,
            ..self
        }
    }

    pub fn load_and_count_pages<U, C>(self, conn: &C) -> QueryResult<Page<U>>
    where
        U: Serialize,
        C: Connection,
        Self: LoadQuery<C, (U, i64)>,
    {
        let per_page = self.per_page;
        let page_number = self.page_number;
        let results = self.load::<(U, i64)>(conn)?;
        let total = results.get(0).map(|x| x.1).unwrap_or(0);
        let records = results.into_iter().map(|x| x.0).collect();
        let page_count = (total as f64 / per_page as f64).ceil() as i64;
        Ok(Page::new(records, page_count, page_number + 1 < page_count))
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T, C> RunQueryDsl<C> for Paginated<T> {}

impl<T> QueryFragment<Pg> for Paginated<T>
where
    T: QueryFragment<Pg>,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("SELECT *, COUNT(*) OVER () FROM (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") t LIMIT ");
        out.push_bind_param::<BigInt, _>(&self.per_page)?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset)?;
        Ok(())
    }
}
