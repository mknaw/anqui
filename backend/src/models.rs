use super::schema::*;
use super::*;
use chrono::prelude::*;
use chrono::Duration;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

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

impl Session {
    pub fn get_current(conn: &PgConnection, try_token: &str) -> Option<Self> {
        use super::schema::sessions::dsl::*;

        // TODO prolly could have this in config / env
        let min_ts = Utc::now().naive_utc() - Duration::hours(36);
        sessions
            .filter(token.eq(try_token))
            .filter(created.gt(min_ts))
            // TODO also must have valid timestamp
            .first::<Session>(conn)
            .ok()
    }

    pub fn create(conn: &PgConnection, user: &User) -> Self {
        use super::schema::sessions::dsl::*;

        let tok: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(25)
            .map(char::from)
            .collect();

        diesel::insert_into(sessions)
            .values((
                user_id.eq(user.id),
                token.eq(tok),
                created.eq(Utc::now().naive_utc()),
            ))
            .get_result(conn)
            .expect("Error saving new card")
    }

    pub fn get_user(&self, conn: &PgConnection) -> User {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(self.user_id))
            .first::<User>(conn)
            .unwrap()
    }
}

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "decks"]
pub struct Deck {
    id: i32,
    name: String,
    user_id: i32,
}

impl Deck {
    pub fn create(conn: &PgConnection, new_deck: NewDeck) -> Deck {
        diesel::insert_into(decks::table)
            .values(&new_deck)
            .get_result(conn)
            .expect("Error saving new deck")
    }
}

#[derive(Deserialize, Insertable, Serialize)]
#[table_name = "decks"]
pub struct NewDeck {
    pub name: String,
}

#[derive(Associations, Identifiable, Queryable, Serialize)]
#[belongs_to(Deck)]
#[table_name = "cards"]
pub struct Card {
    id: i32,
    deck_id: i32,
    front: String,
    back: String,
    revision_weight: i16,
}

#[derive(Deserialize, Insertable, Serialize)]
#[table_name = "cards"]
pub struct NewCard {
    pub front: String,
    pub back: String,
    pub deck_id: i32,
}

impl Card {
    pub fn create(conn: &PgConnection, new_card: NewCard) -> Card {
        diesel::insert_into(cards::table)
            .values(&new_card)
            .get_result(conn)
            .expect("Error saving new card")
    }

    pub fn add_feedback(&mut self, conn: &PgConnection, feedback: &str) {
        // Take user's difficulty rating and change card weight accordingly.
        use super::schema::cards::dsl::*;

        let mut weight = self.revision_weight;
        match feedback {
            "fail" => {
                weight *= 4;
            }
            "hard" => {
                weight *= 2;
            }
            "good" => {
                weight /= 2;
            }
            "easy" => {
                weight /= 4;
            }
            _ => {}
        };
        weight = max(weight, 1);
        weight = min(weight, 32767); // SMALLINT upper bound.
        self.revision_weight = weight;
        diesel::update(cards)
            .filter(id.eq(self.id))
            .set(revision_weight.eq(weight))
            .load::<Card>(conn)
            .unwrap();
    }
}

impl User {
    pub fn new_session(&self, conn: &PgConnection) -> Session {
        use super::schema::sessions::dsl::*;

        diesel::delete(sessions.filter(user_id.eq(self.id)))
            .execute(conn)
            .unwrap();
        Session::create(conn, self)
    }
}
