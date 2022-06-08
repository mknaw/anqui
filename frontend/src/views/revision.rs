use std::fmt;

use common::models::{Deck, RevisionCard};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::AppContext;
use crate::AppRoute;

#[derive(PartialEq, Properties)]
pub struct RevisionProps {
    pub deck_id: i32,
}

#[function_component(Revision)]
pub fn revision(RevisionProps { deck_id }: &RevisionProps) -> Html {
    let card_queue = use_state(|| None);
    let revision_length = use_state(|| 0);
    let flipped = use_state(|| false);

    let ctx = use_context::<AppContext>().unwrap();
    api::get_deck(
        *deck_id,
        Box::new(move |fetched_deck: Deck| {
            ctx.set_title.emit(fetched_deck.name);
        }),
    );

    {
        let card_queue = card_queue.clone();
        let revision_length = revision_length.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let card_queue = card_queue.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/revision/", deck_id);
                    if let Ok::<Vec<RevisionCard>, _>(fetched_cards) = api::get(&url).await {
                        revision_length.set(fetched_cards.len());
                        card_queue.set(Some(fetched_cards));
                    };
                });
                || ()
            },
            (),
        );
    }

    let on_card_click = {
        let flipped = flipped.clone();
        Callback::from(move |_| flipped.set(true))
    };

    let on_feedback_click = match &*card_queue {
        Some(cards) => {
            let card_queue = card_queue.clone();
            let cards = cards.clone();
            let flipped = flipped.clone();

            Callback::from(move |feedback: Feedback| {
                let mut cards = cards.clone();
                flipped.set(false);
                let popped = cards.pop();
                card_queue.set(Some(cards));

                if let Some(card) = popped {
                    wasm_bindgen_futures::spawn_local(async move {
                        let url = format!("/api/cards/{}/feedback/", card.id);
                        let payload = serde_json::Value::String(feedback.to_string());
                        api::post_vanilla(&url, payload).await.ok();
                    });
                }
            })
        }
        None => Callback::from(|_| {}),
    };

    if let Some(card_queue) = (*card_queue).clone() {
        let queue_length = (*card_queue).len();
        let card_count = *revision_length - queue_length + 1;
        let card_count_display = format!("{} / {}", card_count, *revision_length);
        match (*card_queue).last() {
            Some(c) => html! {
                <div
                    class={
                        classes!(
                            "h-full", "flex", "flex-col", "justify-center", "items-center",
                            "text-3xl", "portrait:text-6xl"
                        )
                    }
                >
                    {
                        if *flipped {
                            html! {}
                        } else {
                            html! {
                                <div
                                    onclick={ on_card_click.clone() }
                                    class={ classes!("absolute", "w-[100vw]", "h-[85vh]", "top-[5vh]") }
                                >
                                    // Empty div for making it easier to click wherever
                                </div>
                            }
                        }
                    }
                    <div class={ classes!("h-[40vh]", "flex", "items-end") }>
                        <RevisionCardDisplay
                            card={ c.clone() }
                            flipped={ *flipped.clone() }
                        />
                    </div>
                    {
                        if *flipped {
                            html! {
                                <FeedbackBar
                                    onclick={ on_feedback_click.clone() }
                                />
                            }
                        } else {
                            html! {}
                        }
                    }
                    <div class={ classes!("absolute", "bottom-10") }>
                        { card_count_display }
                    </div>
                </div>
            },
            None => html! {
                // All done!
                <Redirect<AppRoute> to={AppRoute::DeckDetail { deck_id: *deck_id }}/>
            },
        }
    } else {
        html! {}
    }
}

#[derive(PartialEq, Properties)]
struct RevisionCardDisplayProps {
    card: RevisionCard,
    flipped: bool,
}

// TODO probably should pull out a common component to use here and in the card list.
#[function_component(RevisionCardDisplay)]
fn revision_card(props: &RevisionCardDisplayProps) -> Html {
    let cursor = if props.flipped {
        "cursor-default"
    } else {
        "cursor-pointer"
    };

    html! {
        <div class={ classes!("flex", "flex-col", "items-center", cursor) }>
            <div class={ "text-center mb-10" }>{ &props.card.first }</div>
            {
                if props.flipped {
                    html! {
                        // TODO this should always take up a fixed height.
                        <div class={ "text-center mb-10" }>{ &props.card.second }</div>
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

impl fmt::Display for Feedback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Feedback::Fail => write!(f, "fail"),
            Feedback::Hard => write!(f, "hard"),
            Feedback::Good => write!(f, "good"),
            Feedback::Easy => write!(f, "easy"),
        }
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
        <div class={ classes!("flex", "flex-row") }>
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
        <button
            onclick={ on_feedback_click }
            class={
                classes!(
                    "flex", "justify-center", "w-32", "portrait:w-48", "p-5", "mx-5", color,
                    "rounded-lg"
                )
            }
        >
            { &props.feedback.to_string() }
        </button>
    }
}
