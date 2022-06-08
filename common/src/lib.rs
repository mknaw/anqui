#[macro_use]
extern crate diesel;

use std::fmt;

use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

pub mod models;
pub mod query_params;
pub mod schema;

#[derive(DbEnum, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[DieselType = "Flip_mode"]
pub enum FlipMode {
    Front,
    Back,
    Both,
}

impl fmt::Debug for FlipMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlipMode::Front => write!(f, "front"),
            FlipMode::Back => write!(f, "back"),
            FlipMode::Both => write!(f, "both"),
        }
    }
}
