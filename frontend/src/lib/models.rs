// TODO ideally would be a `common` pkg between this and backend.
// Can't figure out how to only have backend care about ORM stuff though.

use serde::Deserialize;

#[derive(Clone, PartialEq, Deserialize)]
pub struct Card {
    pub id: usize,
    pub front: String,
    pub back: String,
}
