extern crate anqui;
extern crate diesel;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anqui::models::*;
use anqui::*;
use self::diesel::prelude::*;
use serde_json::json;

#[get("/cards/")]
async fn read_cards() -> impl Responder {
    use anqui::schema::cards::dsl::*;

    let connection = establish_connection();
    let results = cards
        .limit(5)
        .load::<Card>(&connection)
        .expect("Error loading cards");

    HttpResponse::Ok().json(results)
}

#[post("/cards/new/")]
async fn new_card(new_card: web::Json<NewCard>) -> impl Responder {
    let card = Card::create(new_card.into_inner());
    HttpResponse::Ok().json(card)
}

// fn main() {

    // println!("Displaying {} cards", results.len());
    // for card in results {
        // println!("{}", card.front);
        // println!("----------\n");
        // println!("{}", card.back);
    // }
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(read_cards)
            .service(new_card)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
