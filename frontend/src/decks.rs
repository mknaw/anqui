use reqwasm::http::Request;
use serde::Deserialize;
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::cards::Card;
use crate::routes::Route;

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
        <table id={ "decks" }>
            {
                (*decks).clone().into_iter().map(|deck| {
                    html!{ <DeckListRow { deck } /> }
                }).collect::<Html>()
            }
        </table>
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
                // TODO what if it fails?
                Request::delete(&url).send().await.unwrap();
                hidden.set(true);
            });
        })
    };
    let class = if *hidden { "hidden" } else { "" };
    html! {
        <tr key={ deck.id } { class }>
            <td class={ "emoji" }>
                <Link<Route> to={ Route::DeckDetail { id: deck.id } }>
                    { "⚙️" }
                </Link<Route>>
            </td>
            <td class={ "emoji" }>
                <span onclick={ delete_deck }>
                    { "🪓" }
                </span>
            </td>
            <td class={ "emoji" }>
                <Link<Route> to={ Route::Revision { id: deck.id } }>
                    { "🛎️" }
                </Link<Route>>
            </td>
            <td>{ &deck.name }</td>
        </tr>
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
                    let new_deck: Deck = Request::post("/api/decks/")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&json!({ "name": name })).unwrap())
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    push_deck.emit(new_deck);
                    input.set_value("");
                });
            };
        })
    };

    html! {
        <div class="input-and-button-container">
            <button onclick={ on_add_click }>
                { "✏️" }
            </button>
            <input ref={ input_node_ref } />
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
                    let fetched_cards: Vec<Card> = Request::get(&url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    cards.set(fetched_cards);
                });
                || ()
            },
            (),
        );
    }

    // TODO this stuff is too verbose, must figure out a way to split out to a helper.
    let deck = use_state(|| None);
    {
        let deck = deck.clone();
        let id = id.clone();
        use_effect_with_deps(
            move |_| {
                let deck = deck.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/", id);
                    let fetched_deck: Deck = Request::get(&url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    deck.set(Some(fetched_deck));
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
        Callback::from(move |_| history.push(Route::Revision { id }))
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
                let card: Card = Request::post(&url)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&payload).unwrap())
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                front_input.set_value("");
                back_input.set_value("");
                let mut card_vec = (*cards).clone();
                card_vec.push(card);
                cards.set(card_vec);
            });
        })
    };

    html! {
        <>
            {
                if let Some(deck) = (*deck).clone() {
                    html! {
                        <h1>
                            <span onclick={ on_revise_click.clone() }>{ "🛎️ " }</span>
                            <span>{ deck.name }</span>
                        </h1>
                    }
                } else {
                    html! {}
                }
            }
            <table>
                {
                    (*cards).clone().into_iter().map(|card| {
                        html! {
                            <tr>
                                <td>{ card.front }</td>
                                <td class="card-back">{ card.back }</td>
                            </tr>
                        }
                    }).collect::<Html>()
                }
            </table>
            <div class="input-and-button-container">
                <button onclick={ on_add_click }>
                    { "✏️" }
                </button>
                <div>
                    <input ref={ front_node_ref } placeholder="de face" />
                    <input ref={ back_node_ref } placeholder="arrière" />
                </div>
            </div>
        </>
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
                    let fetched_decks: Vec<Deck> = Request::get("/api/decks/")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    decks.set(fetched_decks);
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
        <>
            <DeckList decks={ (*decks).clone() } />
            <DeckAdd { push_deck } />
        </>
    }
}
