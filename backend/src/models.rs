use super::schema::*;
use super::*;
use chrono::Duration;
use chrono::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

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
    pub id: i32,
    pub name: String,
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
    pub id: i32,
    pub deck_id: i32,
    pub front: String,
    pub back: String,
    pub fail_count: i16,
    pub hard_count: i16,
    pub good_count: i16,
    pub easy_count: i16,
}

#[derive(Deserialize, Insertable, Serialize)]
#[table_name = "cards"]
pub struct NewCard {
    pub front: String,
    pub back: String,
    pub deck_id: i32,
}

#[derive(AsChangeset)]
#[table_name = "cards"]
pub struct PostFeedback {
    pub fail_count: Option<i16>,
    pub hard_count: Option<i16>,
    pub good_count: Option<i16>,
    pub easy_count: Option<i16>,
}

impl Card {
    pub fn create(conn: &PgConnection, new_card: NewCard) -> Card {
        diesel::insert_into(cards::table)
            .values(&new_card)
            .get_result(conn)
            .expect("Error saving new card")
    }
}

impl User {
    pub fn new_session(&self, conn: &PgConnection) -> Session {
        use super::schema::sessions::dsl::*;

        diesel::delete(sessions.filter(user_id.eq(self.id))).execute(conn).unwrap();
        Session::create(conn, &self)
    }
}
