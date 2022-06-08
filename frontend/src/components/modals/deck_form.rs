use common::models::Deck;
use common::FlipMode;
use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use super::Modal;
use crate::api;
use crate::emojis;
use crate::AppContext;

#[derive(PartialEq, Properties)]
pub struct DeckFormModalProps {
    pub deck: Deck,
    pub update_deck: Callback<Deck>,
}

#[function_component(DeckFormModal)]
pub fn deck_form_modal(DeckFormModalProps { deck, update_deck }: &DeckFormModalProps) -> Html {
    let ctx = use_context::<AppContext>().unwrap();

    let name = use_state_eq(|| deck.name.clone());
    let on_name_input = {
        let name = name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            name.set(input.value());
        })
    };

    let revision_length = use_state_eq(|| deck.revision_length);
    let on_revision_length_input = {
        let revision_length = revision_length.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            revision_length.set(input.value().parse::<i16>().unwrap());
        })
    };

    let flip_mode = use_state_eq(|| deck.flip_mode);
    let on_flip_mode_input = {
        let flip_mode = flip_mode.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            match &*input.value() {
                "front" => flip_mode.set(FlipMode::Front),
                "back" => flip_mode.set(FlipMode::Back),
                "both" => flip_mode.set(FlipMode::Both),
                _ => (),
            }
        })
    };

    let onsubmit = {
        let deck_id = deck.id;
        let name = name.clone();
        let revision_length = revision_length.clone();
        let flip_mode = flip_mode.clone();
        let update_deck = update_deck.clone();
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
            let update_deck = update_deck.clone();
            let ctx = ctx.clone();
            let url = format!("/api/decks/{}/", deck_id);
            if name.is_empty() {
                return;
            }
            let payload = json!({
                "name": *name,
                "revision_length": *revision_length,
                "flip_mode": *flip_mode,
            });
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok::<Deck, _>(deck) = api::post(&url, payload).await {
                    update_deck.emit(deck);
                    ctx.set_modal.emit(None);
                }
            });
        })
    };

    html! {
        <Modal title={ Some("Modifier le paquet") }>
            <form { onsubmit }>
                <div class={ classes!("flex", "flex-col") }>
                    <div class={ classes!("pb-4") }>
                        <input
                            oninput={ on_name_input }
                            type="text"
                            value={ (*name).clone() }
                            placeholder={ "Nom du paquet" }
                        />
                    </div>
                    <div class={ classes!("flex", "flex-row", "pb-4") }>
                        <input
                            oninput={ on_revision_length_input }
                            type="range"
                            max={ 25 }
                            min={ 5 }
                            class={ classes!("w-full", "mr-4") }
                        />
                        <span class={ classes!("text-center") }>
                            { *revision_length }
                        </span>
                    </div>
                    <div class={ classes!("w-full", "flex", "justify-between", "text-2xl", "items-center") }>
                        // TODO get `checked` from backend.
                        <span>
                            <input
                                name="flip-mode"
                                type="radio"
                                value="front"
                                checked={ *flip_mode.clone() == FlipMode::Front }
                                oninput={ on_flip_mode_input.clone() }
                            />
                            <label for="front">{ "de face" }</label>
                        </span>
                        <span>
                            <input
                                name="flip-mode"
                                type="radio"
                                value="back"
                                checked={ *flip_mode.clone() == FlipMode::Back }
                                oninput={ on_flip_mode_input.clone() }
                            />
                            <label for="back">{ "arrière" }</label>
                        </span>
                        <span>
                            <input
                                name="flip-mode"
                                type="radio"
                                value="both"
                                checked={ *flip_mode.clone() == FlipMode::Both }
                                oninput={ on_flip_mode_input }
                            />
                            <label for="back">{ "les deux" }</label>
                        </span>
                    </div>
                    <button
                        type={ "submit" }
                        class={ classes!("text-right") }
                    >
                        { emojis::PENCIL }
                    </button>
                </div>
            </form>
        </Modal>
    }
}
