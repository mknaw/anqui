use reqwasm::http::Request;
use serde::Deserialize;
use yew::prelude::*;

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
        <div id={ "card_display" } class={ "bordered" }>
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
        <div id="feedback">
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
        <button class={ "feedback-btn bordered" } onclick={ on_feedback_click }>
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
    let card_queue = use_state(|| vec![]);
    let front_shown = use_state(|| true);

    {
        let card_queue = card_queue.clone();
        let id = id.clone();
        use_effect_with_deps(move |_| {
            let card_queue = card_queue.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/decks/{}/cards/", id);
                let fetched_cards: Vec<Card> = Request::get(&url)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                card_queue.set(fetched_cards);
            });
            || ()
        }, ());
    }

    let on_card_click = {
        let front_shown = front_shown.clone();
        Callback::from(move |_| front_shown.set(false))
    };

    let on_feedback_click = {
        let card_queue = card_queue.clone();
        let cards = (*card_queue).clone();
        let front_shown = front_shown.clone();
        Callback::from(move |feedback: Feedback| {
            let mut cards = cards.clone();
            let front_shown_val = !*front_shown;
            front_shown.set(front_shown_val);
            if !front_shown_val {
                return;
            }
            let popped = cards.pop();
            card_queue.set(cards);

            popped.map(|card: Card| {
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/cards/{}/feedback/", card.id);
                    Request::post(url.as_str())
                        .body(label_feedback(&feedback))
                        .send()
                        .await
                        .unwrap();
                });
            });
        })
    };

    html! {
        <div id={ "revision" }>
            {
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
                        <div>{ "TODO redirect" }</div>
                    },
                }
            }
        </div>
    }
}
