#[macro_use]
extern crate log;

extern crate backend;
extern crate diesel;

use self::diesel::prelude::*;
use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use backend::models::*;
use backend::*;
use dotenv::dotenv;
use std::env;

#[get("/cards/")]
async fn read_cards() -> impl Responder {
    use backend::schema::cards::dsl::*;

    let conn = establish_connection();
    let results = cards
        .load::<Card>(&conn)
        .expect("Error loading cards");

    info!("/cards/ GET");
    HttpResponse::Ok().json(results)
}

#[post("/cards/new/")]
async fn new_card(new_card: web::Json<NewCard>) -> impl Responder {
    let card = Card::create(new_card.into_inner());
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

    if feedback.eq("fail") {
        info!("and it matched the conditional");
    }

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
        let cors = Cors::permissive(); // temporary
        App::new().wrap(cors)
            .service(read_cards)
            .service(new_card)
            .service(post_feedback)
    })
    .bind((host, port.parse::<u16>().unwrap()))?
    .run()
    .await
}
