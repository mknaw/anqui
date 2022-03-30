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

// TODO could really interpret all POSTs (or PUTs maybe) as "new" instead of having URL
#[post("new/")]
async fn new_deck(
    _auth: Authenticated,
    pool: web::Data<DbPool>,
    new_deck: web::Json<NewDeck>,
) -> impl Responder {
    let conn = pool.get().unwrap();
    let deck = Deck::create(&conn, new_deck.into_inner());
    HttpResponse::Ok().json(deck)
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

#[post("/{id}/cards/new/")]
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
    let card = cards.filter(id.eq(card_id)).first::<Card>(&conn).unwrap();

    // TODO would be good I guess to have a less janky way to do this,
    // although I guess it is not the end of the world.
    let changes = PostFeedback {
        fail_count: if feedback == "fail" {
            Some(card.fail_count + 1)
        } else {
            None
        },
        hard_count: if feedback == "hard" {
            Some(card.hard_count + 1)
        } else {
            None
        },
        good_count: if feedback == "good" {
            Some(card.good_count + 1)
        } else {
            None
        },
        easy_count: if feedback == "easy" {
            Some(card.easy_count + 1)
        } else {
            None
        },
    };

    diesel::update(cards)
        .filter(id.eq(card.id))
        .set(&changes)
        .load::<Card>(&conn)
        .unwrap();

    HttpResponse::Ok().body("ok")
}
