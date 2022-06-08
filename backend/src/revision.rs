use std::cmp::{max, min};

use common::models::{Card, RevisionCard};
use common::FlipMode;
use diesel::prelude::*;
use rand::Rng;

pub fn make_revision_card(card: &Card, flip_mode: FlipMode) -> RevisionCard {
    let flip = match flip_mode {
        FlipMode::Front => false,
        FlipMode::Back => true,
        FlipMode::Both => {
            let mut rng = rand::thread_rng();
            rng.gen()
        }
    };
    let first;
    let second;
    if flip {
        first = card.back.clone();
        second = card.front.clone();
    } else {
        first = card.front.clone();
        second = card.back.clone();
    }
    RevisionCard {
        id: card.id,
        deck_id: card.deck_id,
        first,
        second,
    }
}

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
