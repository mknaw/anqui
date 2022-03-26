table! {
    cards (id) {
        id -> Int4,
        deck_id -> Int4,
        front -> Text,
        back -> Text,
        fail_count -> Int2,
        hard_count -> Int2,
        good_count -> Int2,
        easy_count -> Int2,
    }
}

table! {
    decks (id) {
        id -> Int4,
        name -> Text,
    }
}

joinable!(cards -> decks (deck_id));

allow_tables_to_appear_in_same_query!(
    cards,
    decks,
);
