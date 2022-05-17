use std::cmp::{max, min};

use common::models::Card;
use diesel::prelude::*;

pub fn add_feedback(conn: &PgConnection, card: &mut Card, feedback: &str) {
    // Take user's difficulty rating and change card weight accordingly.
    use common::schema::cards::dsl::*;

    let mut weight = card.revision_weight;
    match feedback {
        "fail" => {
            weight *= 4;
        }
        "hard" => {
            weight *= 2;
        }
        "good" => {
            weight /= 2;
        }
        "easy" => {
            weight /= 4;
        }
        _ => {}
    };
    weight = max(weight, 1);
    weight = min(weight, 32767); // SMALLINT upper bound.
    card.revision_weight = weight;
    diesel::update(cards)
        .filter(id.eq(card.id))
        .set(revision_weight.eq(weight))
        .load::<Card>(conn)
        .unwrap();
}
