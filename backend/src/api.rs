use actix_web::{delete, get, post, web, HttpResponse, Responder};
use common::models::{Card, Deck};
use diesel::dsl::{exists, select, sql, sql_query};
use diesel::prelude::*;
use serde::Deserialize;

use crate::auth::Authenticated;
use crate::db::*;
use crate::revision::*;

#[derive(Deserialize)]
struct PageQuery {
    #[serde(default)]
    page: i64,
    #[serde(default = "default_per_page")]
    per_page: i64,
}

fn default_per_page() -> i64 {
    // TODO should come from some (variable) constant?
    20
}

#[get("/")]
async fn read_decks(auth: Authenticated, pool: web::Data<DbPool>) -> impl Responder {
    use common::schema::decks::dsl::*;

    let conn = pool.get().unwrap();
    let results = decks
        .filter(user_id.eq(auth.get_user(&conn).id))
        .load::<Deck>(&conn)
        .unwrap();

    HttpResponse::Ok().json(results)
}

#[derive(Deserialize)]
pub struct DeckPayload {
    pub name: String,
}

#[post("/")]
async fn new_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    new_deck: web::Json<DeckPayload>,
) -> impl Responder {
    use common::schema::decks;

    let conn = pool.get().unwrap();
    let name = new_deck.name.trim();
    let deck = diesel::insert_into(decks::table)
        .values((
            decks::name.eq(name),
            decks::user_id.eq(auth.get_user(&conn).id),
        ))
        .get_result::<Deck>(&conn)
        .unwrap();
    HttpResponse::Ok().json(deck)
}

#[get("{id}/")]
async fn read_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use common::schema::decks::dsl::*;

    let (deck_id,) = path.into_inner();
    let conn = pool.get().unwrap();
    let deck = decks
        .filter(id.eq(deck_id))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .first::<Deck>(&conn)
        .unwrap();

    HttpResponse::Ok().json(deck)
}

#[delete("{id}/")]
async fn delete_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use common::schema::decks::dsl::*;

    let (deck_id,) = path.into_inner();
    let conn = pool.get().unwrap();
    let target = decks
        .filter(id.eq(deck_id))
        .filter(user_id.eq(auth.get_user(&conn).id));
    diesel::delete(target).execute(&conn).unwrap();

    HttpResponse::Ok()
}

#[get("{id}/cards/")]
async fn read_cards(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    query: web::Query<PageQuery>,
) -> impl Responder {
    use common::schema::{cards, decks};

    let (deck_id,) = path.into_inner();
    let conn = pool.get().unwrap();
    let user_id = auth.get_user(&conn).id;
    let page: Page<Card> = cards::table
        .inner_join(decks::table)
        .filter(cards::deck_id.eq(deck_id))
        .filter(decks::user_id.eq(user_id))
        .select(cards::table::all_columns())
        .paginate(query.page, query.per_page)
        .load_and_count_pages(&conn)
        .unwrap();

    HttpResponse::Ok().json(page)
}

#[get("/{deck_id}/cards/{card_id}/")]
async fn read_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    use common::schema::{cards, decks};

    let (deck_id, card_id) = path.into_inner();
    let conn = pool.get().unwrap();
    let user_id = auth.get_user(&conn).id;

    let results: Card = cards::table
        .inner_join(decks::table)
        .filter(cards::id.eq(card_id))
        .filter(cards::deck_id.eq(deck_id))
        .filter(decks::user_id.eq(user_id))
        .select(cards::table::all_columns())
        .first::<Card>(&conn)
        .unwrap();

    HttpResponse::Ok().json(results)
}

#[derive(Deserialize)]
struct CardPayload {
    front: String,
    back: String,
}

#[post("/{deck_id}/cards/{card_id}/")]
async fn update_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
    payload: web::Json<CardPayload>,
) -> impl Responder {
    let (deck_id, card_id) = path.into_inner();
    let payload = payload.into_inner();

    let conn = pool.get().unwrap();
    let user_id = auth.get_user(&conn).id;

    // Diesel does not seem to support this type of query at the time:
    // https://github.com/diesel-rs/diesel/issues/1478
    // Maybe the ORM haters are onto something.
    // (For our purposes could have really just fetched the card / deck to check.)
    let update_query = format!(
        r#"
        UPDATE cards
        SET 
            front = '{}',
            back = '{}'
        FROM decks
        WHERE
            cards.deck_id = decks.id
            AND cards.id = {}
            AND decks.id = {}
            AND decks.user_id = {};
    "#,
        payload.front.trim(),
        payload.back.trim(),
        card_id,
        deck_id,
        user_id
    );
    sql_query(update_query).execute(&conn).unwrap();

    HttpResponse::Ok()
}

#[post("/{id}/cards/")]
async fn new_card(
    auth: Authenticated, // TODO
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    payload: web::Json<CardPayload>,
) -> impl Responder {
    use common::schema::{cards, decks};

    let (deck_id,) = path.into_inner();
    let payload = payload.into_inner();
    let conn = pool.get().unwrap();

    let deck_query = decks::table
        .filter(decks::id.eq(deck_id))
        .filter(decks::user_id.eq(auth.get_user(&conn).id));
    let valid_deck = select(exists(deck_query)).get_result(&conn).unwrap();

    if valid_deck {
        let card: Card = diesel::insert_into(cards::table)
            .values((
                cards::front.eq(&payload.front),
                cards::back.eq(&payload.back),
                cards::deck_id.eq(&deck_id),
            ))
            .get_result(&conn)
            .unwrap();
        HttpResponse::Ok().json(card)
    } else {
        HttpResponse::BadRequest().finish()
    }
}

#[delete("/{deck_id}/cards/{card_id}/")]
async fn delete_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (deck_id, card_id) = path.into_inner();
    let conn = pool.get().unwrap();
    let user_id = auth.get_user(&conn).id;

    // https://github.com/diesel-rs/diesel/issues/1478
    let delete_query = format!(
        r#"
        DELETE FROM cards
        USING decks
        WHERE
            cards.deck_id = decks.id
            AND decks.id = {}
            AND cards.id = {}
            AND decks.user_id = {};
    "#,
        deck_id, card_id, user_id
    );
    sql_query(delete_query).execute(&conn).unwrap();

    HttpResponse::Ok()
}

#[post("/cards/{id}/feedback/")]
async fn post_feedback(
    _auth: Authenticated, // TODO
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    feedback: String,
) -> impl Responder {
    use common::schema::cards::dsl::*;

    let conn = pool.get().unwrap();
    let card_id = path.into_inner().0;
    // TODO only need to select `revision_weight` really
    // TODO probably best to assert this is from a deck of the right user.
    let mut card = cards.filter(id.eq(card_id)).first::<Card>(&conn).unwrap();
    add_feedback(&conn, &mut card, &feedback);

    HttpResponse::Ok().body("ok")
}

#[get("{id}/revision/")]
async fn get_revision_cards(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use common::schema::cards::dsl::*;
    use common::schema::decks::dsl::{decks, user_id};

    let conn = pool.get().unwrap();
    let ids = cards
        // TODO it's not really that great to get all of Deck when
        // just want to know that it's a deck of the right `user_id`.
        .inner_join(decks)
        .filter(deck_id.eq(path.into_inner().0))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .order_by(sql::<i32>("random() ^ revision_weight"))
        .select(id)
        // TODO should come from a user preference.
        .limit(10)
        .load::<i32>(&conn)
        .unwrap();

    let results = cards
        .filter(id.eq_any(ids))
        .order_by(sql::<i32>("random()"))
        .load::<Card>(&conn)
        .unwrap();

    HttpResponse::Ok().json(results)
}
