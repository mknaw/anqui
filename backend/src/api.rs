use super::auth::Authenticated;
use super::db::DbPool;
use super::diesel::prelude::*;
use super::models::*;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::Deserialize;

#[get("/")]
async fn read_decks(_auth: Authenticated, pool: web::Data<DbPool>) -> impl Responder {
    use super::schema::decks::dsl::*;

    let conn = pool.get().unwrap();
    let results = decks.load::<Deck>(&conn).expect("Error loading decks");

    HttpResponse::Ok().json(results)
}

#[post("/")]
async fn new_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    // TODO at the moment this `NewDeck` struct is a little contrived
    new_deck: web::Json<NewDeck>,
) -> impl Responder {
    use super::schema::decks::dsl::*;

    let conn = pool.get().unwrap();
    let deck = diesel::insert_into(decks)
        .values((
            name.eq(&new_deck.name),
            user_id.eq(auth.get_user(&conn).id)
        ))
        .get_result::<Deck>(&conn)
        .unwrap();
    HttpResponse::Ok().json(deck)
}

#[delete("{id}/")]
async fn delete_deck(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::decks::dsl::*;

    let conn = pool.get().unwrap();
    diesel::delete(decks.filter(id.eq(path.into_inner().0)))
        .execute(&conn)
        .expect("Error deleting deck");

    HttpResponse::Ok()
}

#[get("{id}/cards/")]
async fn read_cards(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    let conn = pool.get().unwrap();
    let results = cards
        .filter(deck_id.eq(path.into_inner().0))
        .load::<Card>(&conn)
        .expect("Error loading cards");

    HttpResponse::Ok().json(results)
}

// TODO don't love the duplication between here and NewCard, and even Card to some extent
#[derive(Deserialize)]
struct NewWebCard {
    front: String,
    back: String,
}

#[post("/{id}/cards/")]
async fn new_card(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    payload: web::Json<NewWebCard>,
) -> impl Responder {
    let payload = payload.into_inner();
    let card = Card::create(
        &pool.get().unwrap(),
        NewCard {
            deck_id: path.into_inner().0,
            front: payload.front,
            back: payload.back,
        },
    );
    HttpResponse::Ok().json(card)
}

#[post("/cards/{id}/feedback/")]
async fn post_feedback(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    feedback: String,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    let conn = pool.get().unwrap();
    let card_id = path.into_inner().0;
    // TODO only need to select `revision_weight` really
    let mut card = cards.filter(id.eq(card_id)).first::<Card>(&conn).unwrap();
    card.add_feedback(&conn, &feedback);

    HttpResponse::Ok().body("ok")
}

#[get("{id}/revision/")]
async fn get_revision_cards(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    let conn = pool.get().unwrap();
    let ids = cards
        .filter(deck_id.eq(path.into_inner().0))
        .order_by(diesel::dsl::sql::<i32>("random() ^ revision_weight"))
        // TODO would be nice to take this from a user preference.
        .select(id)
        .limit(25)
        .load::<i32>(&conn)
        .expect("Error loading cards");

    let results = cards
        .filter(id.eq_any(ids))
        .order_by(diesel::dsl::sql::<i32>("random()"))
        .load::<Card>(&conn)
        .expect("Error loading cards");

    HttpResponse::Ok().json(results)
}
