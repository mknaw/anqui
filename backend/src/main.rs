#[macro_use]
extern crate log;

extern crate backend;
extern crate diesel;

use self::diesel::prelude::*;
use actix_files::{Files, NamedFile};
use actix_web::*;
use backend::models::*;
use backend::*;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

async fn index(_data: web::Path<()>) -> impl Responder {
    NamedFile::open_async("./frontend/dist/index.html").await
}

#[get("/")]
async fn read_decks() -> impl Responder {
    use backend::schema::decks::dsl::*;

    let conn = establish_connection();
    let results = decks
        .load::<Deck>(&conn)
        .expect("Error loading decks");

    info!("/decks/ GET");
    HttpResponse::Ok().json(results)
}

#[delete("{id}/")]
async fn delete_deck(path: web::Path<(i32,)>) -> impl Responder {
    use backend::schema::decks::dsl::*;

    let conn = establish_connection();
    diesel::delete(decks.filter(id.eq(path.into_inner().0)))
        .execute(&conn)
        .expect("Error deleting deck");

    HttpResponse::Ok()
}

// TODO could really interpret all POSTs (or PUTs maybe) as "new" instead of having URL
#[post("new/")]
async fn new_deck(new_deck: web::Json<NewDeck>) -> impl Responder {
    info!("/decks/new/ POST");
    let deck = Deck::create(new_deck.into_inner());
    HttpResponse::Ok().json(deck)
}

#[get("{id}/cards/")]
async fn read_cards(path: web::Path<(i32,)>) -> impl Responder {
    use backend::schema::cards::dsl::*;

    let conn = establish_connection();
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
async fn new_card(path: web::Path<(i32,)>, payload: web::Json<NewWebCard>) -> impl Responder {
    let payload = payload.into_inner();
    info!("{} {}", payload.front, payload.back);
    let card = Card::create(NewCard {
        deck_id: path.into_inner().0,
        front: payload.front,
        back: payload.back,
    });
    HttpResponse::Ok().json(card)
}

#[post("/cards/{id}/feedback/")]
async fn post_feedback(path: web::Path<(i32,)>, feedback: String) -> impl Responder {
    use backend::schema::cards::dsl::*;

    let conn = establish_connection();
    let card_id = path.into_inner().0;
    let card = cards
        .filter(id.eq(card_id))
        .first::<Card>(&conn)
        .unwrap();

    // TODO would be good I guess to have a less janky way to do this,
    // although I guess it is not the end of the world.
    let changes = PostFeedback {
        fail_count: if feedback == "fail" {Some(card.fail_count + 1)} else {None},
        hard_count: if feedback == "hard" {Some(card.hard_count + 1)} else {None},
        good_count: if feedback == "good" {Some(card.good_count + 1)} else {None},
        easy_count: if feedback == "easy" {Some(card.easy_count + 1)} else {None},
    };

    diesel::update(cards)
        .filter(id.eq(card.id))
        .set(&changes)
        .load::<Card>(&conn)
        .unwrap();

    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST not set.");
    let port = env::var("PORT").expect("PORT not set.");

    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/decks")
                            .service(read_decks)
                            .service(new_deck)
                            .service(delete_deck)
                            .service(read_cards)
                            .service(new_card)
                    )
                    .service(post_feedback)
            )
            .service(Files::new("/", "frontend/dist/").index_file("index.html"))
            .default_service(web::get().to(index))
    })
    .bind((host, port.parse::<u16>().unwrap()))?
    .run()
    .await
}
