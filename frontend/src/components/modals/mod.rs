use yew::prelude::*;

pub mod card_form;
pub mod deck_form;

pub(crate) use card_form::CardFormModal;
pub(crate) use deck_form::DeckFormModal;

#[derive(PartialEq, Properties)]
pub struct ModalProps {
    pub children: Children,
    pub title: Option<String>,
}

#[function_component(Modal)]
pub fn modal(ModalProps { children, title }: &ModalProps) -> Html {
    html! {
        <>
            <div
                class={
                    classes!(
                        "z-50", "bg-stone-800", "text-5xl", "px-8",
                        "rounded-lg",
                        "portrait:text-6xl", "text-3xl"
                    )
                }
            >
                {
                    if let Some(title) = title {
                        html! {
                            <div class={ classes!("w-full", "text-center", "pt-8") }>
                                { title }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                <div class={ classes!("py-8") }>
                    { for children.iter() }
                </div>
            </div>
        </>
    }
}
