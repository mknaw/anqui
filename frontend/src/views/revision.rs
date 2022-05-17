use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::models::*;
use crate::AppContext;
use crate::AppRoute;

#[derive(PartialEq, Properties)]
pub struct RevisionProps {
    pub deck_id: usize,
}

#[function_component(Revision)]
pub fn revision(RevisionProps { deck_id }: &RevisionProps) -> Html {
    let card_queue: UseStateHandle<Option<Vec<Card>>> = use_state(|| None);
    let front_shown = use_state(|| true);

    let ctx = use_context::<AppContext>().unwrap();
    api::get_deck(
        *deck_id,
        Box::new(move |fetched_deck: Deck| {
            ctx.set_title.emit(fetched_deck.name);
        }),
    );

    {
        let card_queue = card_queue.clone();
        let deck_id = *deck_id;
        use_effect_with_deps(
            move |_| {
                let card_queue = card_queue.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let url = format!("/api/decks/{}/revision/", deck_id);
                    if let Ok::<Vec<Card>, _>(fetched_cards) = api::get(&url).await {
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
                        api::post_vanilla(&url, payload).await.ok();
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
