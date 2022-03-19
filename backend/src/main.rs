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

    let connection = establish_connection();
    let results = cards
        // .limit(1)
        .load::<Card>(&connection)
        .expect("Error loading cards");

    info!("/cards/ GET");
    HttpResponse::Ok().json(results)
}

#[post("/cards/new/")]
async fn new_card(new_card: web::Json<NewCard>) -> impl Responder {
    let card = Card::create(new_card.into_inner());
    HttpResponse::Ok().json(card)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST not set.");
    let port = env::var("PORT").expect("PORT not set.");

    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::permissive(); // temporary
        App::new().wrap(cors).service(read_cards).service(new_card)
    })
    .bind((host, port.parse::<u16>().unwrap()))?
    .run()
    .await
}
