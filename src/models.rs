use serde::{Deserialize, Serialize};
use super::*;
use super::schema::cards;

#[derive(Queryable, Serialize)]
pub struct Card {
    pub id: i32,
    pub front: String,
    pub back: String,
}

#[derive(Deserialize, Insertable, Serialize)]
#[table_name = "cards"]
pub struct NewCard {
    pub front: String,
    pub back: String,
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
            .expect("Error saving new post")
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
