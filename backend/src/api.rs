use super::auth::Authenticated;
use super::db::DbPool;
use super::diesel::prelude::*;
use super::models::*;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use serde::Deserialize;

#[get("/")]
async fn read_decks(auth: Authenticated, pool: web::Data<DbPool>) -> impl Responder {
    use super::schema::decks::dsl::*;

    log::info!("read_decks");

    let conn = pool.get().unwrap();
    let results = decks
        .filter(user_id.eq(auth.get_user(&conn).id))
        .load::<Deck>(&conn)
        .expect("Error loading decks");

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
        .values((name.eq(&new_deck.name), user_id.eq(auth.get_user(&conn).id)))
        .get_result::<Deck>(&conn)
        .unwrap();
    HttpResponse::Ok().json(deck)
}

#[delete("{id}/")]
async fn delete_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::decks::dsl::*;

    let conn = pool.get().unwrap();
    let target = decks
        .filter(id.eq(path.into_inner().0))
        .filter(user_id.eq(auth.get_user(&conn).id));
    diesel::delete(target)
        .execute(&conn)
        .expect("Error deleting deck");

    HttpResponse::Ok()
}

#[get("{id}/")]
async fn read_deck(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::decks::dsl::*;

    log::info!("read_deck");

    let conn = pool.get().unwrap();
    let deck = decks
        .filter(id.eq(path.into_inner().0))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .first::<Deck>(&conn)
        .expect("Error getting deck");

    HttpResponse::Ok().json(deck)
}

#[get("{id}/cards/")]
async fn read_cards(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;
    use super::schema::decks::dsl::{decks, user_id};

    log::info!("read_cards");

    let conn = pool.get().unwrap();
    let results: Vec<Card> = cards
        // TODO it's not really that great to get all of Deck when
        // just want to know that it's a deck of the right `user_id`.
        .inner_join(decks)
        .filter(deck_id.eq(path.into_inner().0))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .load(&conn)
        .expect("Error loading cards")
        .into_iter()
        .map(|(c, _): (Card, Deck)| c)
        .collect();

    HttpResponse::Ok().json(results)
}

#[get("/{deck_id}/cards/{card_id}/")]
async fn read_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;
    use super::schema::decks::dsl::{decks, user_id};

    log::info!("read_card");
    let (this_deck_id, this_card_id) = path.into_inner();

    let conn = pool.get().unwrap();
    let results: Card = cards
        // TODO it's not really that great to get all of Deck when
        // just want to know that it's a deck of the right `user_id`.
        .inner_join(decks)
        .filter(id.eq(this_card_id))
        .filter(deck_id.eq(this_deck_id))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .first::<(Card, Deck)>(&conn)
        .map(|(c, _)| c)
        .expect("Error loading cards");

    HttpResponse::Ok().json(results)
}

// TODO don't love the duplication between here and NewCard, and even Card to some extent
#[derive(Deserialize)]
struct NewWebCard {
    front: String,
    back: String,
}

#[post("/{deck_id}/cards/{card_id}/")]
async fn update_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
    payload: web::Json<NewWebCard>,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    log::info!("update_card");
    // TODO still have to confirm this is the right user and the right deck.
    let (this_deck_id, this_card_id) = path.into_inner();
    let payload = payload.into_inner();

    log::info!("{}", payload.back);

    let conn = pool.get().unwrap();

    let target = cards.filter(id.eq(this_card_id));
    let result_count = diesel::update(target)
        .set((
            front.eq(payload.front),
            back.eq(payload.back),
        ))
        .execute(&conn)
        .expect("Error updating card");

    log::info!("...successfully?");
    log::info!("results: {}", result_count);
    HttpResponse::Ok()
}

#[post("/{id}/cards/")]
async fn new_card(
    _auth: Authenticated, // TODO
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    payload: web::Json<NewWebCard>,
) -> impl Responder {
    let payload = payload.into_inner();
    let card = Card::create(
        &pool.get().unwrap(),
        NewCard {
            // TODO probably best to assert this is from a deck of the right user.
            deck_id: path.into_inner().0,
            front: payload.front,
            back: payload.back,
        },
    );
    HttpResponse::Ok().json(card)
}

#[delete("/{deck_id}/cards/{card_id}/")]
async fn delete_card(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    // TODO still have to confirm this is the right user and the right deck.
    let (this_deck_id, this_card_id) = path.into_inner();

    let conn = pool.get().unwrap();
    let target = cards.filter(id.eq(this_card_id));
    diesel::delete(target)
        .execute(&conn)
        .expect("Error deleting card");

    HttpResponse::Ok()
}

#[post("/cards/{id}/feedback/")]
async fn post_feedback(
    _auth: Authenticated, // TODO
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
    feedback: String,
) -> impl Responder {
    use super::schema::cards::dsl::*;

    let conn = pool.get().unwrap();
    let card_id = path.into_inner().0;
    // TODO only need to select `revision_weight` really
    // TODO probably best to assert this is from a deck of the right user.
    let mut card = cards.filter(id.eq(card_id)).first::<Card>(&conn).unwrap();
    card.add_feedback(&conn, &feedback);

    HttpResponse::Ok().body("ok")
}

#[get("{id}/revision/")]
async fn get_revision_cards(
    auth: Authenticated,
    pool: web::Data<DbPool>,
    path: web::Path<(i32,)>,
) -> impl Responder {
    use super::schema::cards::dsl::*;
    use super::schema::decks::dsl::{decks, user_id};

    let conn = pool.get().unwrap();
    let ids = cards
        // TODO it's not really that great to get all of Deck when
        // just want to know that it's a deck of the right `user_id`.
        .inner_join(decks)
        .filter(deck_id.eq(path.into_inner().0))
        .filter(user_id.eq(auth.get_user(&conn).id))
        .order_by(diesel::dsl::sql::<i32>("random() ^ revision_weight"))
        .select(id)
        // TODO would be nice to take this from a user preference.
        .limit(5)
        .load::<i32>(&conn)
        .expect("Error loading cards");

    let results = cards
        .filter(id.eq_any(ids))
        .order_by(diesel::dsl::sql::<i32>("random()"))
        .load::<Card>(&conn)
        .expect("Error loading cards");

    HttpResponse::Ok().json(results)
}
