use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::*;

#[derive(Identifiable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(User)]
#[table_name = "sessions"]
pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub token: String,
    pub created: NaiveDateTime,
}

#[derive(Clone, PartialEq, Identifiable, Queryable, Deserialize, Serialize)]
#[table_name = "decks"]
pub struct Deck {
    pub id: i32,
    pub name: String,
    pub user_id: i32,
    pub revision_length: i16,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "decks"]
pub struct PostDeck {
    pub name: Option<String>,
    pub revision_length: Option<i16>,
}

#[derive(Clone, PartialEq, Associations, Identifiable, Queryable, Deserialize, Serialize)]
#[belongs_to(Deck)]
pub struct Card {
    pub id: i32,
    pub deck_id: i32,
    pub front: String,
    pub back: String,
    pub revision_weight: i16,
}
