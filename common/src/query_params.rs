use serde::{Deserialize, Serialize};

fn default_per_page() -> i64 {
    // TODO gold plate would be to get this from config
    36
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct CardReadQuery {
    // TODO would be nice to figure out how to reuse these bits across any paginated URL params
    #[serde(default)]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
    #[serde(default)]
    pub search_term: String,
}

impl Default for CardReadQuery {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: default_per_page(),
            search_term: String::new(),
        }
    }
}
