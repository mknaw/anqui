use serde::Deserialize;
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::{delete, get, post};
use crate::cards::Card;
use crate::routes::AppRoute;
use crate::utils::truncate;

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
        <div class={ classes!("text-3xl") }>
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
    let delete_deck = {
        let hidden = hidden.clone();
        let deck = deck.clone();
        Callback::from(move |_| {
            let hidden = hidden.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/decks/{}/", deck.id);
                match delete(&url).await {
                    Ok(_) => hidden.set(true),
                    Err(_) => (), // TODO probably should do... something...
                }
            });
        })
    };
    let hidden_class = if *hidden { "hidden" } else { "" };
    html! {
        <div key={ deck.id } class={ classes!("py-2", hidden_class) }>
            <span class={ classes!("px-2") }>
                <span onclick={ delete_deck }>
                    { "🪓" }
                </span>
            </span>
            <span class={ classes!("px-2") }>
                <Link<AppRoute> to={ AppRoute::Revision { id: deck.id } }>
                    { "🛎️" }
                </Link<AppRoute>>
            </span>
            <span class={ classes!("px-2") }>
                <Link<AppRoute> to={ AppRoute::DeckDetail { id: deck.id } }>
                    { &deck.name }
                </Link<AppRoute>>
            </span>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DeckAddProps {
    push_deck: Callback<Deck>,
}

#[function_component(DeckAdd)]
pub fn deck_add(DeckAddProps { push_deck }: &DeckAddProps) -> Html {
    let input_node_ref = use_node_ref();

    let on_add_click = {
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
                    match post("/api/decks/", payload).await {
                        Ok::<Deck, _>(new_deck) => {
                            push_deck.emit(new_deck);
                            input.set_value("");
                        }
                        _ => (),
                    }
                });
            };
        })
    };

    html! {
        <div class={ classes!("text-3xl", "flex", "w-full", "py-3") }>
            <button onclick={ on_add_click } class={ classes!("px-2") }>
                { "✏️" }
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
    pub id: usize,
}

#[function_component(DeckDetail)]
pub fn deck_detail(DeckDetailProps { id }: &DeckDetailProps) -> Html {
    let front_node_ref = use_node_ref();
    let back_node_ref = use_node_ref();

    let cards = use_state(|| vec![]);
    {
        let cards = cards.clone();
        let id = id.clone();
        use_effect_with_deps(
            move |_| {
                let cards = cards.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/cards/", id);
                    match get(&url).await {
                        Ok::<Vec<Card>, _>(fetched_cards) => {
                            cards.set(fetched_cards);
                        }
                        _ => (),
                    }
                });
                || ()
            },
            (),
        );
    }

    let deck = use_state(|| None);
    {
        let deck = deck.clone();
        let id = id.clone();
        use_effect_with_deps(
            move |_| {
                let deck = deck.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/", id);
                    match get(&url).await {
                        Ok::<Deck, _>(fetched_deck) => {
                            deck.set(Some(fetched_deck));
                        }
                        _ => (),
                    }
                });
                || ()
            },
            (),
        );
    }

    let history = use_history().unwrap();
    let on_revise_click = {
        let id = id.clone();
        let history = history.clone();
        Callback::from(move |_| history.push(AppRoute::Revision { id }))
    };

    let on_add_click = {
        let id = id.clone();
        let front_node_ref = front_node_ref.clone();
        let back_node_ref = back_node_ref.clone();
        let cards = cards.clone();

        Callback::from(move |_| {
            let id = id.clone();
            // let push_deck = push_deck.clone();
            let front_input = front_node_ref.cast::<HtmlInputElement>().unwrap();
            let back_input = back_node_ref.cast::<HtmlInputElement>().unwrap();
            let front = front_input.value();
            let back = back_input.value();
            let cards = cards.clone();
            if front.is_empty() || back.is_empty() {
                // TODO some sort of warning? maybe not needed
                return;
            }
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/decks/{}/cards/", id);
                let payload = json!({ "front": front, "back": back });
                match post(&url, payload).await {
                    Ok::<Card, _>(card) => {
                        front_input.set_value("");
                        back_input.set_value("");
                        let mut card_vec = (*cards).clone();
                        card_vec.push(card);
                        cards.set(card_vec);
                    }
                    _ => (),
                }
            });
        })
    };

    html! {
        <div class={ classes!("container", "max-w-4xl", "h-full") }>
            {
                if let Some(deck) = (*deck).clone() {
                    html! {
                        <h1 class={ classes!("text-4xl", "text-center", "py-2") }>
                            <span onclick={ on_revise_click.clone() }>{ "🛎️ " }</span>
                            <span>{ deck.name }</span>
                        </h1>
                    }
                } else {
                    html! {}
                }
            }
            <div class={ classes!("grid", "gap-4", "grid-cols-4", "py-4") }>
                {
                    (*cards).clone().into_iter().map(|card| {
                        html! {
                            <div class={
                                classes!(
                                    "h-32", "flex", "flex-col", "justify-between", "items-center",
                                    "text-l", "text-center",
                                    "rounded-lg", "border-2", "border-gray-600",
                                )
                            } >
                                <CardContent content={ card.front } />
                                <hr class={ classes!("w-full", "border-gray-600", "border", "border-dashed") } />
                                <CardContent content={ card.back } />
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
            <div>
                <button onclick={ on_add_click }>
                    { "✏️" }
                </button>
                <div>
                    <input ref={ front_node_ref } placeholder="de face" />
                    <input ref={ back_node_ref } placeholder="arrière" />
                </div>
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct CardContentProps {
    content: String,
}

#[function_component(CardContent)]
fn card_content(CardContentProps { content }: &CardContentProps) -> Html {
    html! {
        <span class={ classes!("py-1", "h-full", "flex", "flex-col", "justify-center") }>
            { truncate(content, 45) }
        </span>
    }
}

#[function_component(DeckHome)]
pub fn deck_home() -> Html {
    let decks = use_state(|| vec![]);
    {
        let decks = decks.clone();
        use_effect_with_deps(
            move |_| {
                let decks = decks.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match get("/api/decks/").await {
                        Ok::<Vec<Deck>, _>(fetched_decks) => {
                            decks.set(fetched_decks);
                        }
                        _ => (),
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
        <div class={ classes!("container", "max-w-2xl", "h-3/5") }>
            <DeckList decks={ (*decks).clone() } />
            <DeckAdd { push_deck } />
        </div>
    }
}
