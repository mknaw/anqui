table! {
    cards (id) {
        id -> Int4,
        deck_id -> Int4,
        front -> Text,
        back -> Text,
        revision_weight -> Int2,
    }
}

table! {
    decks (id) {
        id -> Int4,
        name -> Text,
        user_id -> Int4,
    }
}

table! {
    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        token -> Text,
        created -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Text,
        password -> Text,
    }
}

joinable!(cards -> decks (deck_id));
joinable!(decks -> users (user_id));

allow_tables_to_appear_in_same_query!(
    cards,
    decks,
    sessions,
    users,
);
