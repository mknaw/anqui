use serde::Deserialize;
use serde_json::json;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::{delete, get, post_vanilla};
use crate::AppRoute;

#[derive(Clone, PartialEq, Deserialize)]
pub struct Card {
    pub id: usize,
    pub front: String,
    pub back: String,
}

#[derive(PartialEq, Properties)]
struct CardDisplayProps {
    card: Card,
    front_shown: bool,
    onclick: Callback<()>,
}

#[function_component(CardDisplay)]
fn cards(props: &CardDisplayProps) -> Html {
    let display = if props.front_shown {
        props.card.front.to_string()
    } else {
        props.card.back.to_string()
    };
    let foo = {
        let onclick = props.onclick.clone();
        Callback::from(move |_| onclick.emit(()))
    };

    html! {
        <div>
            <div onclick={ foo }>{ display }</div>
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
        <div>
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

    html! {
        <button onclick={ on_feedback_click }>
            { label_feedback(&props.feedback) }
        </button>
    }
}

#[derive(PartialEq, Properties)]
pub struct RevisionProps {
    pub id: usize,
}

#[function_component(Revision)]
pub fn revision(RevisionProps { id }: &RevisionProps) -> Html {
    let card_queue: UseStateHandle<Option<Vec<Card>>> = use_state(|| None);
    let front_shown = use_state(|| true);

    {
        let card_queue = card_queue.clone();
        let id = id.clone();
        use_effect_with_deps(
            move |_| {
                let card_queue = card_queue.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/cards/", id);
                    match get(&url).await {
                        Ok::<Vec<Card>, _>(fetched_cards) => card_queue.set(Some(fetched_cards)),
                        Err(_) => (),
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

                popped.map(|card: Card| {
                    wasm_bindgen_futures::spawn_local(async move {
                        let url = format!("/api/cards/{}/feedback/", card.id);
                        let payload = serde_json::Value::String(label_feedback(&feedback));
                        match post_vanilla(&url, payload).await {
                            Ok(_) => (),
                            Err(_) => (),
                        };
                    });
                });
            })
        }
        None => Callback::from(|_| return),
    };

    html! {
        <div>
            {
                if let Some(card_queue) = (*card_queue).clone() {
                    match (*card_queue).last() {
                        Some(c) => html! {
                            <>
                                <CardDisplay
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
                            </>
                        },
                        None => html!{
                            <Redirect<AppRoute> to={AppRoute::DeckDetail { id: id.clone() }}/>
                        },
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct CardDetailProps {
    pub deck_id: usize,
    pub card_id: usize,
}

#[function_component(CardDetail)]
pub fn card_detail(CardDetailProps { deck_id, card_id }: &CardDetailProps) -> Html {

    let api_url = format!("/api/decks/{}/cards/{}/", deck_id, card_id);
    let history = use_history().unwrap();
    // TODO if accessing from the view in which we already got all the cards as a list,
    // should just be able to pass that serialized data `Option`ally.
    let front = use_state(|| "".to_string());
    let back = use_state(|| "".to_string());
    {
        let front = front.clone();
        let back = back.clone();
        let api_url = api_url.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    match get(&api_url).await {
                        Ok::<Card, _>(fetched_card) => {
                            front.set(fetched_card.front.clone());
                            back.set(fetched_card.back.clone());
                        }
                        _ => (),
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
        let deck_id = deck_id.clone();
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
                match post_vanilla(&api_url, payload).await {
                    Ok(_) => history.push(AppRoute::DeckDetail { id: deck_id.clone() }),
                    Err(_) => (),  // TODO
                }
            });
        })
    };

    let delete = {
        let api_url = api_url.clone();
        let deck_id = deck_id.clone();
        let history = history.clone();

        Callback::from(move |_| {
            let history = history.clone();
            let api_url = api_url.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api_url = api_url.clone();
                match delete(&api_url).await {
                    Ok(_) => history.push(AppRoute::DeckDetail { id: deck_id.clone() }),
                    Err(_) => (),  // TODO
                }
            });
        })
    };

    html! {
        <div>
            <form { onsubmit } class={ classes!("flex", "flex-col") }>
                <textarea
                    value={ (*front).clone() }
                    onchange={ on_front_change }
                />
                <textarea
                    value={ (*back).clone() }
                    onchange={ on_back_change }
                />
                <button type={ "submit" }>
                    { "submit" }
                </button>
            </form>
            <button onclick={ delete }>
                { "🪓" }
            </button>
        </div>
    }
}
