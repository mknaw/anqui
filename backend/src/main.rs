#[macro_use]
extern crate diesel;
extern crate dotenv;

extern crate bcrypt;
extern crate log;

use std::env;

use actix_files::{Files, NamedFile};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware::ErrorHandlers, web, App, HttpServer, Responder};
use dotenv::dotenv;

use crate::api::*;
use crate::auth::*;
use crate::db::new_db_pool;

mod api;
mod auth;
mod db;
mod revision;

async fn index(_auth: Authenticated, _data: web::Path<()>) -> impl Responder {
    // Need to "default" serve `index.html` from every random URL to play nice with Yew routes.
    NamedFile::open_async("./frontend/dist/index.html").await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("HOST").expect("HOST not set.");
    let port = env::var("PORT").expect("PORT not set.");

    env_logger::init();

    HttpServer::new(move || {
        let policy = CookieIdentityPolicy::new(&[0; 32])
            .name("auth-cookie")
            .secure(false);

        App::new()
            .app_data(web::Data::new(new_db_pool()))
            .wrap(
                ErrorHandlers::new().handler(http::StatusCode::UNAUTHORIZED, redirect_on_autherror),
            )
            .wrap(AuthenticateMiddlewareFactory::default())
            .wrap(IdentityService::new(policy))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/decks")
                            .service(read_decks)
                            .service(read_deck)
                            .service(new_deck)
                            .service(delete_deck)
                            .service(read_cards)
                            .service(read_card)
                            .service(new_card)
                            .service(update_card)
                            .service(delete_card)
                            .service(get_revision_cards),
                    )
                    .service(post_feedback),
            )
            .service(login_get)
            .service(login)
            .service(logout)
            .service(Files::new("/static/", "frontend/dist/").index_file("index.html"))
            .default_service(web::get().to(index))
    })
    .bind((host, port.parse::<u16>().unwrap()))?
    .run()
    .await
}
