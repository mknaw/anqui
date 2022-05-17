use common::models::{Card, Deck};
use common::query_params::CardReadQuery;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::emojis;
use crate::routes::AppRoute;
use crate::AppContext;

#[derive(PartialEq, Properties)]
pub struct DeckDetailProps {
    pub deck_id: i32,
}

#[function_component(DeckDetail)]
pub fn deck_detail(DeckDetailProps { deck_id }: &DeckDetailProps) -> Html {
    let query_params = use_state_eq(CardReadQuery::default);

    let on_search_term_input = {
        let query_params = query_params.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_query_params = (*query_params).clone();
            new_query_params.page = 0;
            new_query_params.search_term = input.value();
            query_params.set(new_query_params);
        })
    };

    // Fetch list of cards associated with this deck
    let cards = use_state_eq(std::vec::Vec::new);
    let has_more = use_mut_ref(|| false);
    {
        let cards = cards.clone();
        let query_params = query_params.clone();
        let has_more = has_more.clone();
        let this_page = query_params.page;
        let deck_id = *deck_id;

        let url = format!(
            "/api/decks/{}/cards/?{}",
            deck_id,
            serde_qs::to_string(&*query_params).unwrap(),
        );
        use_effect_with_deps(
            move |_| {
                let cards = cards.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok::<api::Page<Card>, _>(page) = api::get(&url).await {
                        // Would prefer not to have to clone all the old ones to append
                        // but I increasingly thing that would necessitate something `unsafe`.
                        if this_page > 0 {
                            let mut updated_cards = (*cards).clone();
                            updated_cards.extend(page.results);
                            cards.set(updated_cards);
                        } else {
                            cards.set(page.results);
                        }
                        *has_more.borrow_mut() = page.has_more;
                    }
                });
                || ()
            },
            query_params, // TODO would be nice to cache by `query_params`.
        );
    }

    // Fetch info about deck.
    let deck = use_state_eq(|| None);
    let ctx = use_context::<AppContext>().unwrap();
    {
        let deck = deck.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let deck = deck.clone();
                api::get_deck(
                    deck_id,
                    Box::new(move |fetched_deck| {
                        ctx.set_title.emit(fetched_deck.name.clone());
                        deck.set(Some(fetched_deck));
                    }),
                );
                || ()
            },
            (),
        );
    }

    let onscroll = {
        let query_params = query_params.clone();
        Callback::from(move |e: Event| {
            if !*has_more.borrow_mut() {
                return;
            }
            let el = e.target().unwrap().unchecked_into::<Element>();
            // TODO adding 5 arbitrarily still seems kind of flimsy.
            if el.scroll_top() + el.client_height() + 5 >= el.scroll_height() {
                let mut new_query_params = (*query_params).clone();
                new_query_params.page += 1;
                query_params.set(new_query_params);
            };
        })
    };

    html! {
        <>
            <div class={ classes!("w-1/3", "h-[5vh]", "flex", "items-center", "portrait:text-4xl", "text-l") }>
                <input
                    class={ classes!("w-full", "text-center") }
                    type="text"
                    placeholder={ "Chercher" }
                    value={ (*query_params).clone().search_term }
                    oninput={ on_search_term_input }
                />
            </div>
            <div { onscroll } class={ classes!("h-[85vh]", "w-full", "overflow-y-auto", "px-24") }>
                <div class={ classes!("max-h-[85vh]", "grid", "gap-4", "portrait:grid-cols-3", "grid-cols-4") }>
                    {
                        (*cards).clone().into_iter().map(|card| {
                            html! { <CardSummary deck_id={ *deck_id } card={ card.clone() } /> }
                        }).collect::<Html>()
                    }
                </div>
            </div>
            {
                if let Some(deck) = (*deck).clone() {
                    html! { <DeckDetailToolbar { deck } /> }
                } else {
                    html! {}
                }
            }
        </>
    }
}

#[derive(PartialEq, Properties)]
struct DeckDetailToolbarProps {
    deck: Deck,
}

#[function_component(DeckDetailToolbar)]
fn deck_detail_toolbar(DeckDetailToolbarProps { deck }: &DeckDetailToolbarProps) -> Html {
    let history = use_history().unwrap();
    let on_revise_click = {
        let history = history.clone();
        let deck_id = deck.id;
        Callback::from(move |_| history.push(AppRoute::Revision { deck_id }))
    };

    let on_create_click = {
        let history = history;
        let deck_id = deck.id;
        Callback::from(move |_| {
            history.push(AppRoute::CardCreateForm { deck_id });
        })
    };

    html! {
        <div
            class={
                classes!(
                    "sticky", "bottom-0", "h-[5vh]", "w-1/3", "z-10",
                    "flex", "justify-between", "items-center",
                    "portrait:text-6xl", "text-3xl",
                )
            }
        >
            <button
                onclick={ on_revise_click.clone() }
                class={ classes!("px-2") }
            >
                { emojis::BELL }
            </button>
            <button
                onclick={ on_create_click }
                class={ classes!("px-2") }
            >
                { emojis::PENCIL }
            </button>
            <button
                // TODO onclick={ ... }
                class={ classes!("px-2") }
            >
                { emojis::GEAR }
            </button>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct CardSummaryProps {
    deck_id: i32,
    card: Card,
}

#[function_component(CardSummary)]
fn card(CardSummaryProps { deck_id, card }: &CardSummaryProps) -> Html {
    fn card_content(content: &str) -> Html {
        html! {
            <span
                class={
                    classes!(
                        "p-1", "h-full",  "w-full", "overflow-hidden", "flex", "items-center", "justify-center"
                    )
                }
            >
                // TODO figure out some dynamic way to truncate / clip?
                { content }
            </span>
        }
    }

    let history = use_history().unwrap();
    let onclick = {
        let deck_id = *deck_id;
        let card_id = card.id;
        let history = history;
        Callback::from(move |_| {
            history.push(AppRoute::CardUpdateForm { deck_id, card_id });
        })
    };

    html! {
        <div
            class={
                classes!(
                    "h-32", "portrait:h-52", "flex", "flex-col", "justify-between", "items-center",
                    "text-l", "portrait:text-4xl", "text-center",
                    "rounded-lg", "border-2", "border-gray-600",
                    "bg-black", "cursor-pointer",
                    "transition", "hover:bg-gray-800", "duration-300",
                )
            }
            { onclick }
            key={ card.id }
        >
            { card_content(&card.front) }
            <hr class={ classes!("w-full", "border-gray-600", "border", "border-dashed") } />
            { card_content(&card.back) }
        </div>
    }
}
