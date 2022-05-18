use common::models::Deck;
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::components::modals::DeckFormModal;
use crate::emojis;
use crate::routes::AppRoute;
use crate::AppContext;

#[function_component(DeckList)]
pub fn deck_list() -> Html {
    let decks = use_state(Vec::new);
    {
        let decks = decks.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok::<Vec<Deck>, _>(fetched_decks) = api::get("/api/decks/").await {
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
            let mut deck_vec = (*decks).clone();
            deck_vec.push(deck);
            decks.set(deck_vec);
        })
    };

    html! {
        <div class={ classes!("max-w-2xl", "h-3/5") }>
            <div class={ classes!("text-6xl", "lg:text-3xl") }>
                {
                    (*decks).clone().into_iter().map(|deck| {
                        html!{ <DeckListRow { deck } /> }
                    }).collect::<Html>()
                }
            </div>
            <DeckCreate { push_deck } />
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct DeckListRowProps {
    deck: Deck,
}

#[function_component(DeckListRow)]
fn deck_list_row(DeckListRowProps { deck }: &DeckListRowProps) -> Html {
    let deck = use_state(|| deck.clone());
    let hidden = use_state(|| false);
    let on_delete = {
        let hidden = hidden.clone();
        let deck = deck.clone();
        Callback::from(move |_| {
            let hidden = hidden.clone();
            let deck = deck.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/decks/{}/", deck.id);
                if api::delete(&url).await.is_ok() {
                    hidden.set(true);
                };
            });
        })
    };

    // Allow children to update the `deck` state.
    let update_deck = {
        let deck = deck.clone();
        Callback::from(move |updated_deck: Deck| {
            deck.set(updated_deck);
        })
    };

    let ctx = use_context::<AppContext>().unwrap();
    let on_gear_click = {
        let deck = (*deck).clone();
        Callback::from(move |_| {
            let deck = deck.clone();
            let update_deck = update_deck.clone();
            ctx.set_modal.emit(Some(html! {
                <DeckFormModal { deck } { update_deck } />
            }));
        })
    };

    let deck_id = deck.id;
    html! {
        <div key={ deck.id } hidden={ *hidden } class={ classes!("py-2") }>
            <button onclick={ on_delete } class={ classes!("px-2") }>
                { emojis::AXE }
            </button>
            // TODO these should probably just be buttons as well
            <span class={ classes!("px-2") }>
                <Link<AppRoute> to={ AppRoute::Revision { deck_id } }>
                    { emojis::BELL }
                </Link<AppRoute>>
            </span>
            <span class={ classes!("px-2") }>
                <button onclick={ on_gear_click }>
                    { emojis::GEAR }
                </button>
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

#[function_component(DeckCreate)]
pub fn deck_create(DeckCreateProps { push_deck }: &DeckCreateProps) -> Html {
    let input_node_ref = use_node_ref();

    let ctx = use_context::<AppContext>().unwrap();
    ctx.set_title.emit("Paquets".to_string());

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
                    if let Ok::<Deck, _>(new_deck) = api::post("/api/decks/", payload).await {
                        push_deck.emit(new_deck);
                        input.set_value("");
                    }
                });
            };
        })
    };

    html! {
        // TODO pencil + input should just be a single component.
        <div class={ classes!("text-3xl", "portrait:text-6xl", "flex", "w-full", "py-3") }>
            <button onclick={ on_create } class={ classes!("px-2") }>
                { emojis::PENCIL }
            </button>
            <input
                ref={ input_node_ref }
                placeholder={ "Nouveau paquet" }
                class={ classes!("w-full") }
            />
        </div>
    }
}
