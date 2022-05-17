use crate::cards::*;
use crate::decks::*;
use crate::login::*;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/login/")]
    Login,
    #[at("/")]
    Decks,
    #[at("/decks/:id")]
    DeckDetail { id: usize },
    #[at("/decks/:id/revision/")]
    Revision { id: usize },
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Login => html! { <Login /> },
        Route::Decks => html! { <DeckHome /> },
        Route::DeckDetail { id } => html! { <DeckDetail id={ id.clone() } /> },
        Route::Revision { id } => html! { <Revision id={ id.clone() }/> },
        Route::NotFound => html! {
            <>
                <h1>{ "Page not found 🤕" }</h1>
                <Link<Route> to={ Route::Decks }>{ "Take me home" }</Link<Route>>
            </>
        },
    }
}
