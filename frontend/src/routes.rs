use yew::prelude::*;
use yew_router::prelude::*;

use crate::cards::*;
use crate::decks::*;
use crate::login::*;
use crate::Layout;

#[derive(Clone, Routable, PartialEq)]
pub enum MainRoute {
    #[at("")]
    RedirectToHome,
    #[at("/login/")]
    Login,
    #[at("/app/*")]
    AppRoot,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, Routable, PartialEq)]
pub enum AppRoute {
    #[at("/app/decks/")]
    Decks,
    // TODO would be nice to have a title slug instead of int id.
    #[at("/app/decks/:id/")]
    DeckDetail { id: usize },
    #[at("/app/decks/:id/revision/")]
    Revision { id: usize },
    #[at("/app/decks/:deck_id/cards/:card_id/")]
    CardDetail { deck_id: usize, card_id: usize },
}

pub fn main_switch(routes: &MainRoute) -> Html {
    match routes {
        MainRoute::RedirectToHome => html! { <Redirect<AppRoute> to={AppRoute::Decks}/> },
        MainRoute::Login => html! { <Login /> },
        MainRoute::NotFound => html! {
            <>
                <h1>{ "Page not found 🤕" }</h1>
                <Link<AppRoute> to={ AppRoute::Decks }>{ "🏡" }</Link<AppRoute>>
            </>
        },
        MainRoute::AppRoot => html! {
            <Layout>
                <Switch<AppRoute> render={Switch::render(app_switch)} />
            </Layout>
        },
    }
}

fn app_switch(routes: &AppRoute) -> Html {
    match routes {
        AppRoute::Decks => html! { <DeckHome /> },
        AppRoute::DeckDetail { id } => html! { <DeckDetail id={ *id } /> },
        AppRoute::CardDetail { deck_id, card_id } => html! {
            <CardDetail deck_id={ *deck_id } card_id={ *card_id }/>
        },
        AppRoute::Revision { id } => html! { <Revision id={ *id }/> },
    }
}
