#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod api;
pub mod auth;
pub mod db;
pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
