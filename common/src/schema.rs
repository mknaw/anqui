table! {
    use diesel::sql_types::*;
    use crate::*;

    cards (id) {
        id -> Int4,
        deck_id -> Int4,
        front -> Text,
        back -> Text,
        revision_weight -> Int2,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::*;

    decks (id) {
        id -> Int4,
        name -> Text,
        user_id -> Int4,
        revision_length -> Int2,
        flip_mode -> Flip_mode,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::*;

    sessions (id) {
        id -> Int4,
        user_id -> Int4,
        token -> Text,
        created -> Timestamp,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::*;

    users (id) {
        id -> Int4,
        username -> Text,
        password -> Text,
    }
}

joinable!(cards -> decks (deck_id));
joinable!(decks -> users (user_id));

allow_tables_to_appear_in_same_query!(cards, decks, sessions, users,);
