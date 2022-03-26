use super::schema::*;
use super::*;
use serde::{Deserialize, Serialize};

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "decks"]
pub struct Deck {
    pub id: i32,
    pub name: String,
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
    // pub fn find_all() -> Result<Vec<Self>, ApiError> {
    // let conn = db::connection()?;

    // let users = user::table
    // .load::<User>(&conn)?;

    // Ok(users)
    // }

    // pub fn find(id: Uuid) -> Result<Self, ApiError> {
    // let conn = db::connection()?;

    // let user = user::table
    // .filter(user::id.eq(id))
    // .first(&conn)?;

    // Ok(user)
    // }

    pub fn create(new_card: NewCard) -> Card {
        let conn = establish_connection();

        diesel::insert_into(cards::table)
            .values(&new_card)
            .get_result(&conn)
            .expect("Error saving new card")
    }

    // pub fn update(id: Uuid, user: UserMessage) -> Result<Self, ApiError> {
    // let conn = db::connection()?;

    // let user = diesel::update(user::table)
    // .filter(user::id.eq(id))
    // .set(user)
    // .get_result(&conn)?;

    // Ok(user)
    // }

    // pub fn delete(id: Uuid) -> Result<usize, ApiError> {
    // let conn = db::connection()?;

    // let res = diesel::delete(
    // user::table
    // .filter(user::id.eq(id))
    // )
    // .execute(&conn)?;

    // Ok(res)
    // }
}

// TODO surely must be a way to genericize.
impl Deck {
    pub fn create(new_deck: NewDeck) -> Deck {
        let conn = establish_connection();

        diesel::insert_into(decks::table)
            .values(&new_deck)
            .get_result(&conn)
            .expect("Error saving new deck")
    }
}
