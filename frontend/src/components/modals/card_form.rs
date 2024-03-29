use common::models::Card;
use serde_json::json;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

use super::Modal;
use crate::api;
use crate::emojis;
use crate::AppContext;
use crate::AppRoute;

#[derive(PartialEq, Properties)]
pub struct CardFormModalProps {
    pub deck_id: i32,
    pub card_id: Option<i32>,
}

#[function_component(CardFormModal)]
pub fn card_form_modal(CardFormModalProps { deck_id, card_id }: &CardFormModalProps) -> Html {
    let ctx = use_context::<AppContext>().unwrap();
    let history = use_history().unwrap();

    let api_url = if let Some(card_id) = card_id {
        format!("/api/decks/{}/cards/{}/", deck_id, card_id)
    } else {
        format!("/api/decks/{}/cards/", deck_id)
    };

    // TODO surely there's a DRYer way to approach this.
    let front = use_state(String::new);
    let back = use_state(String::new);

    // TODO if accessing from the view in which we already got all the cards as a list,
    // should just be able to pass that serialized data `Option`ally.
    if card_id.is_some() {
        let front = front.clone();
        let back = back.clone();
        let api_url = api_url.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok::<Card, _>(fetched_card) = api::get(&api_url).await {
                        front.set(fetched_card.front);
                        back.set(fetched_card.back);
                    }
                });
                || ()
            },
            (),
        );
    }

    let on_front_change = {
        let front = front.clone();
        Callback::from(move |e: Event| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            front.set(textarea.value());
        })
    };

    let on_back_change = {
        let back = back.clone();
        Callback::from(move |e: Event| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            back.set(textarea.value());
        })
    };

    let onsubmit = {
        let api_url = api_url.clone();
        let front = front.clone();
        let back = back.clone();
        let deck_id = *deck_id;
        let ctx = ctx.clone();
        let history = history.clone();
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let ctx = ctx.clone();
            let history = history.clone();
            let api_url = api_url.clone();
            if front.is_empty() || back.is_empty() {
                return;
            }
            let payload = json!({
                "front": *front,
                "back": *back,
            });
            wasm_bindgen_futures::spawn_local(async move {
                let api_url = api_url.clone();
                if api::post_vanilla(&api_url, payload).await.is_ok() {
                    ctx.set_modal.emit(None);
                    // TODO doesn't actually trigger refetch.
                    history.go(0);
                    //history.replace(AppRoute::DeckDetail { deck_id });
                } // TODO else ...
            });
        })
    };

    let on_delete = {
        let deck_id = *deck_id;

        Callback::from(move |_| {
            let ctx = ctx.clone();
            let history = history.clone();
            let api_url = api_url.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api_url = api_url.clone();
                if api::delete(&api_url).await.is_ok() {
                    ctx.set_modal.emit(None);
                    // TODO doesn't actually trigger refetch.
                    history.push(AppRoute::DeckDetail { deck_id });
                } // TODO else ...
            });
        })
    };

    html! {
        <Modal title={ Some("Modifier le paquet") }>
            <form { onsubmit } class={ classes!("flex", "flex-col", "text-3xl", "portrait:text-6xl") }>
                <textarea
                    value={ (*front).clone() }
                    onchange={ on_front_change }
                    placeholder={ "de face" }
                    class={ classes!("h-64") }
                />
                <textarea
                    value={ (*back).clone() }
                    onchange={ on_back_change }
                    placeholder={ "arrière" }
                    class={ classes!("h-64") }
                />
                <div
                    class={
                        classes!(
                            "w-full", "flex", "justify-around", "mt-5", "text-3xl", "portrait:text-6xl",
                        )
                    }
                >
                    {
                        if card_id.is_some() {
                            html! {
                                <button onclick={ on_delete }>
                                    { emojis::AXE }
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <button type={ "submit" }>
                        { emojis::PENCIL }
                    </button>
                </div>
            </form>
        </Modal>
    }
}
