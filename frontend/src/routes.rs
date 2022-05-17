use yew::prelude::*;
use yew_router::prelude::*;

use crate::emojis;
use crate::views::cards::*;
use crate::views::decks::*;
use crate::views::login::*;
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
    #[at("/app/decks/:deck_id/")]
    DeckDetail { deck_id: usize },
    #[at("/app/decks/:deck_id/revision/")]
    Revision { deck_id: usize },
    #[at("/app/decks/:deck_id/cards/")]
    CardCreateForm { deck_id: usize },
    #[at("/app/decks/:deck_id/cards/:card_id/")]
    CardUpdateForm { deck_id: usize, card_id: usize },
}

pub fn main_switch(routes: &MainRoute) -> Html {
    match routes {
        MainRoute::RedirectToHome => html! { <Redirect<AppRoute> to={AppRoute::Decks}/> },
        MainRoute::Login => html! { <Login /> },
        MainRoute::NotFound => html! {
            <>
                <h1>{ format!("Page not found {}", emojis::HEAD_BANDAGE) }</h1>
                <Link<AppRoute> to={ AppRoute::Decks }>{ emojis::HOME }</Link<AppRoute>>
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
        AppRoute::DeckDetail { deck_id } => html! { <DeckDetail deck_id={ *deck_id } /> },
        AppRoute::CardCreateForm { deck_id } => html! {
            <CardForm deck_id={ *deck_id } card_id={ None }/>
        },
        AppRoute::CardUpdateForm { deck_id, card_id } => html! {
            <CardForm deck_id={ *deck_id } card_id={ *card_id }/>
        },
        AppRoute::Revision { deck_id } => html! { <Revision deck_id={ *deck_id }/> },
    }
}
