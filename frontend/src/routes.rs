use crate::cards::*;
use crate::decks::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Decks,
    #[at("/decks/:id")]
    DeckDetail { id: usize },
    #[at("decks/:id/revision/")]
    Revision { id: usize },
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Revision { id } => html! { <Revision id={ id.clone() }/> },
        Route::Decks => html! { <DeckHome /> },
        Route::DeckDetail { id } => html! { <DeckDetail id={ id.clone() } /> },
    }
}
