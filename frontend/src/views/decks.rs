use serde::Deserialize;
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::{delete, get, post, Page};
use crate::emojis;
use crate::models::Card;
use crate::routes::AppRoute;

#[derive(Clone, PartialEq, Deserialize)]
pub struct Deck {
    id: usize,
    name: String,
}

#[derive(PartialEq, Properties)]
pub struct DeckListProps {
    decks: Vec<Deck>,
}

#[function_component(DeckList)]
pub fn decks(DeckListProps { decks }: &DeckListProps) -> Html {
    html! {
        <div class={ classes!("text-6xl", "lg:text-3xl") }>
            {
                (*decks).clone().into_iter().map(|deck| {
                    html!{ <DeckListRow { deck } /> }
                }).collect::<Html>()
            }
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DeckListRowProps {
    deck: Deck,
}

#[function_component(DeckListRow)]
fn deck_list_row(DeckListRowProps { deck }: &DeckListRowProps) -> Html {
    let hidden = use_state(|| false);
    let on_delete = {
        let hidden = hidden.clone();
        let deck = deck.clone();
        Callback::from(move |_| {
            let hidden = hidden.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/decks/{}/", deck.id);
                if delete(&url).await.is_ok() {
                    hidden.set(true);
                };
            });
        })
    };
    let deck_id = deck.id;
    html! {
        <div key={ deck.id } hidden={ *hidden } class={ classes!("py-2") }>
            <span class={ classes!("px-2") }>
                <span onclick={ on_delete }>
                    { emojis::AXE }
                </span>
            </span>
            <span class={ classes!("px-2") }>
                <Link<AppRoute> to={ AppRoute::Revision { deck_id } }>
                    { emojis::BELL }
                </Link<AppRoute>>
            </span>
            <span class={ classes!("px-2") }>
                <Link<AppRoute> to={ AppRoute::DeckDetail { deck_id } }>
                    { &deck.name }
                </Link<AppRoute>>
            </span>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DeckCreateProps {
    push_deck: Callback<Deck>,
}

#[function_component(DeckAdd)]
pub fn deck_create(DeckCreateProps { push_deck }: &DeckCreateProps) -> Html {
    let input_node_ref = use_node_ref();

    let on_create = {
        let input_node_ref = input_node_ref.clone();
        let push_deck = push_deck.clone();

        Callback::from(move |_| {
            let push_deck = push_deck.clone();
            let input = input_node_ref.cast::<HtmlInputElement>();
            if let Some(input) = input {
                let name = input.value();
                if name.is_empty() {
                    // TODO some sort of warning? maybe not needed
                    log::info!("its empty");
                    return;
                }
                wasm_bindgen_futures::spawn_local(async move {
                    let payload = json!({ "name": name });
                    if let Ok::<Deck, _>(new_deck) = post("/api/decks/", payload).await {
                        push_deck.emit(new_deck);
                        input.set_value("");
                    }
                });
            };
        })
    };

    html! {
        <div class={ classes!("text-3xl", "portrait:text-6xl", "flex", "w-full", "py-3") }>
            <button onclick={ on_create } class={ classes!("px-2") }>
                { emojis::PENCIL }
            </button>
            <input
                ref={ input_node_ref }
                placeholder={ "Nouveau paquet de cartes" }
                class={ classes!("w-full") }
            />
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DeckDetailProps {
    pub deck_id: usize,
}

#[function_component(DeckDetail)]
pub fn deck_detail(DeckDetailProps { deck_id }: &DeckDetailProps) -> Html {
    let page_number = use_state(|| 0);

    // Fetch list of cards associated with this deck
    let cards = use_state(std::vec::Vec::new);
    let has_more = use_state(|| false);
    {
        let cards = cards.clone();
        let has_more = has_more.clone();
        let this_page_number = *page_number;
        let page_number = page_number.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let cards = cards.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!(
                        "/api/decks/{}/cards/?page={}&per_page={}",
                        deck_id, this_page_number, 24
                    );
                    if let Ok::<Page<Card>, _>(page) = get(&url).await {
                        cards.set(page.results);
                        has_more.set(page.has_more);
                    }
                });
                || ()
            },
            page_number,
        );
    }

    // Fetch info about deck.
    let deck = use_state(|| None);
    {
        let deck = deck.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let deck = deck.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/", deck_id);
                    if let Ok::<Deck, _>(fetched_deck) = get(&url).await {
                        deck.set(Some(fetched_deck));
                    }
                });
                || ()
            },
            (),
        );
    }

    let on_previous_click = {
        let old_page_number = *page_number;
        let page_number = page_number.clone();
        Callback::from(move |_| {
            page_number.set(std::cmp::max(old_page_number - 1, 0));
        })
    };

    let on_next_click = {
        let old_page_number = *page_number;
        let page_number = page_number.clone();
        Callback::from(move |_| {
            page_number.set(old_page_number + 1);
        })
    };

    html! {
        <>
            {
                if let Some(deck) = (*deck).clone() {
                    html! { <DeckDetailHeader { deck } /> }
                } else {
                    html! {}
                }
            }
            <div class={ classes!("h-full", "w-full") }>
                <div class={ classes!("flex", "flex-row", "items-center", "justify-center", "h-full") }>
                    <div
                        class={
                            classes!(
                                "text-3xl", "portrait:text-6xl", "px-5",
                                if *page_number > 0 {"visible"} else {"invisible"},
                            )
                        }
                    >
                        <button onclick={ on_previous_click }>
                            { emojis::BACK }
                        </button>
                    </div>
                    <div
                        class={
                            classes!(
                                "w-full",
                                "grid", "gap-4",
                                "portrait:grid-cols-3", "grid-cols-4", "lg:grid-cols-5"
                            )
                        }
                    >
                        {
                            // TODO think the solution to resizable grids is capping this iteration
                            // at some number divisible by grid rows.
                            (*cards).clone().into_iter().map(|card| {
                                html! { <CardSummary deck_id={ *deck_id } card={ card.clone() } /> }
                            }).collect::<Html>()
                        }
                    </div>
                    <div
                        class={
                            classes!(
                                "text-3xl", "portrait:text-6xl", "px-5",
                                if *has_more {"visible"} else {"invisible"},
                            )
                        }
                    >
                        <button onclick={ on_next_click }>
                            { emojis::FORWARD }
                        </button>
                    </div>
                </div>
            </div>
        </>
    }
}

#[derive(PartialEq, Properties)]
struct DeckDetailHeaderProps {
    deck: Deck,
}

#[function_component(DeckDetailHeader)]
fn deck_detail_header(DeckDetailHeaderProps { deck }: &DeckDetailHeaderProps) -> Html {
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
        <h1 class={ classes!("lg:text-4xl", "portrait:text-6xl", "text-center", "py-5") }>
            <div>{ deck.name.clone() }</div>
            <div>
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
        </h1>
    }
}

#[derive(PartialEq, Properties)]
pub struct CardSummaryProps {
    deck_id: usize,
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
                // TODO have to figure out some dynamic way to truncate / clip
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
                    "cursor-pointer"
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

#[function_component(DeckHome)]
pub fn deck_home() -> Html {
    let decks = use_state(Vec::new);
    {
        let decks = decks.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok::<Vec<Deck>, _>(fetched_decks) = get("/api/decks/").await {
                        decks.set(fetched_decks);
                    }
                });
                || ()
            },
            (),
        );
    }

    let push_deck = {
        let decks = decks.clone();
        Callback::from(move |deck: Deck| {
            // TODO too stupid at Rust still to figure it out now but should be able
            // to append to a shared deck vec instead of cloning the whole thing?
            let mut deck_vec = (*decks).clone();
            deck_vec.push(deck);
            decks.set(deck_vec);
        })
    };

    html! {
        <div class={ classes!("max-w-2xl", "h-3/5") }>
            <DeckList decks={ (*decks).clone() } />
            <DeckAdd { push_deck } />
        </div>
    }
}
