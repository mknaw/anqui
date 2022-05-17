use serde_json::json;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::AppContext;
use crate::api::{delete, get, get_deck, post_vanilla};
use crate::emojis;
use crate::models::{Card, Deck};
use crate::AppRoute;

#[derive(PartialEq, Properties)]
pub struct CardFormProps {
    pub deck_id: usize,
    pub card_id: Option<usize>,
}

#[function_component(CardForm)]
pub fn card_form(CardFormProps { deck_id, card_id }: &CardFormProps) -> Html {
    let api_url = if let Some(card_id) = card_id {
        format!("/api/decks/{}/cards/{}/", deck_id, card_id)
    } else {
        format!("/api/decks/{}/cards/", deck_id)
    };
    let history = use_history().unwrap();
    // TODO if accessing from the view in which we already got all the cards as a list,
    // should just be able to pass that serialized data `Option`ally.
    let front = use_state(|| "".to_string());
    let back = use_state(|| "".to_string());
    if card_id.is_some() {
        let front = front.clone();
        let back = back.clone();
        let api_url = api_url.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok::<Card, _>(fetched_card) = get(&api_url).await {
                        front.set(fetched_card.front);
                        back.set(fetched_card.back);
                    }
                });
                || ()
            },
            (),
        );
    }

    // TODO surely there's a DRYer way to approach this.
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
        let history = history.clone();
        Callback::from(move |e: FocusEvent| {
            e.prevent_default();
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
                if post_vanilla(&api_url, payload).await.is_ok() {
                    history.push(AppRoute::DeckDetail { deck_id });
                } // TODO else ...
            });
        })
    };

    let on_delete = {
        let deck_id = *deck_id;
        let history = history.clone();

        Callback::from(move |_| {
            let history = history.clone();
            let api_url = api_url.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api_url = api_url.clone();
                if delete(&api_url).await.is_ok() {
                    history.push(AppRoute::DeckDetail { deck_id });
                } // TODO else ...
            });
        })
    };

    let on_return = Callback::from(move |_| {
        history.back();
    });

    html! {
        <div>
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
                <div class={ classes!("flex", "w-full", "justify-around", "text-3xl", "portrait:text-6xl") }>
                    <button type={ "submit" }>
                        { emojis::PENCIL }
                    </button>
                    <button onclick={ on_return }>
                        { emojis::RETURN }
                    </button>
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
                </div>
            </form>
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct RevisionCardDisplayProps {
    card: Card,
    front_shown: bool,
    onclick: Callback<()>,
}

// TODO probably should pull out a common component to use here and in the card list.
#[function_component(RevisionCardDisplay)]
fn revision_card(props: &RevisionCardDisplayProps) -> Html {
    let cursor = props.front_shown.then(|| "cursor-pointer");

    let onclick = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(()))
    };

    html! {
        <div class={ classes!("flex", "flex-col", "items-center", cursor) }>
            <div { onclick }>{ &props.card.front }</div>
            {
                if !props.front_shown {
                    html! {
                        <div>{ &props.card.back }</div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[derive(Clone, PartialEq)]
pub enum Feedback {
    Fail,
    Hard,
    Good,
    Easy,
}

fn label_feedback(feedback: &Feedback) -> String {
    match &feedback {
        Feedback::Fail => "fail".to_string(),
        Feedback::Hard => "hard".to_string(),
        Feedback::Good => "good".to_string(),
        Feedback::Easy => "easy".to_string(),
    }
}

#[derive(PartialEq, Properties)]
struct FeedbackBarProps {
    onclick: Callback<Feedback>,
}

#[function_component(FeedbackBar)]
fn feedback_bar(FeedbackBarProps { onclick }: &FeedbackBarProps) -> Html {
    let feedbacks = [
        Feedback::Fail,
        Feedback::Hard,
        Feedback::Good,
        Feedback::Easy,
    ];

    html! {
        <div
            class={
                classes!(
                    "flex", "flex-row", "mt-5"
                )
            }
        >
            {
                feedbacks.into_iter().map(|feedback| {
                    html!{
                        <FeedbackButton
                            feedback={ feedback }
                            onclick={ onclick.clone() }
                        />
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

#[derive(PartialEq, Properties)]
struct FeedbackButtonProps {
    feedback: Feedback,
    onclick: Callback<Feedback>,
}

#[function_component(FeedbackButton)]
fn feedback_button(props: &FeedbackButtonProps) -> Html {
    let on_feedback_click = {
        let onclick = props.onclick.clone();
        let feedback = props.feedback.clone();
        Callback::from(move |_| onclick.emit(feedback.clone()))
    };

    let color = match props.feedback {
        Feedback::Fail => "bg-red-500",
        Feedback::Hard => "bg-orange-500",
        Feedback::Good => "bg-yellow-500",
        Feedback::Easy => "bg-green-500",
    };

    html! {
        <div
            class={
                classes!(
                    "flex", "justify-center", "w-32", "px-5", color,
                )
            }
        >
            <button onclick={ on_feedback_click }>
                { label_feedback(&props.feedback) }
            </button>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct RevisionProps {
    pub deck_id: usize,
}

#[function_component(Revision)]
pub fn revision(RevisionProps { deck_id }: &RevisionProps) -> Html {
    let card_queue: UseStateHandle<Option<Vec<Card>>> = use_state(|| None);
    let front_shown = use_state(|| true);

    let ctx = use_context::<AppContext>().unwrap();
    get_deck(*deck_id, Box::new(
        move |fetched_deck: Deck| {
            ctx.set_title.emit(fetched_deck.name);
        }
    ));

    {
        let card_queue = card_queue.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let card_queue = card_queue.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/revision/", deck_id);
                    if let Ok::<Vec<Card>, _>(fetched_cards) = get(&url).await {
                        card_queue.set(Some(fetched_cards));
                    };
                });
                || ()
            },
            (),
        );
    }

    let on_card_click = {
        let front_shown = front_shown.clone();
        Callback::from(move |_| front_shown.set(false))
    };

    let on_feedback_click = match &*card_queue {
        Some(cards) => {
            let card_queue = card_queue.clone();
            let cards = cards.clone();
            let front_shown = front_shown.clone();

            Callback::from(move |feedback: Feedback| {
                let mut cards = cards.clone();
                let front_shown_val = !*front_shown;
                front_shown.set(front_shown_val);
                if !front_shown_val {
                    return;
                }
                let popped = cards.pop();
                card_queue.set(Some(cards));

                if let Some(card) = popped {
                    wasm_bindgen_futures::spawn_local(async move {
                        let url = format!("/api/cards/{}/feedback/", card.id);
                        let payload = serde_json::Value::String(label_feedback(&feedback));
                        post_vanilla(&url, payload).await.ok();
                    });
                }
            })
        }
        None => Callback::from(|_| {}),
    };

    if let Some(card_queue) = (*card_queue).clone() {
        match (*card_queue).last() {
            Some(c) => html! {
                <div
                    class={
                        classes!(
                            "h-[60vh]", "flex", "flex-col", "justify-center", "items-center",
                            "text-3xl", "portrait:text-4xl"
                        )
                    }
                >
                    <RevisionCardDisplay
                        card={ c.clone() }
                        front_shown={ *front_shown.clone() }
                        onclick={ on_card_click.clone() }
                    />
                    {
                        if !(*front_shown) {
                            html! {
                                <FeedbackBar
                                    onclick={ on_feedback_click.clone() }
                                />
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            },
            None => html!{
                // All done!
                <Redirect<AppRoute> to={AppRoute::DeckDetail { deck_id: *deck_id }}/>
            },
        }
    } else {
        html! {}
    }
}
