use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ModalProps {
    pub show: bool,
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(ModalProps { show, children }: &ModalProps) -> Html {
    if !show {
        return html! {}
    }

    html! {
        <>
            <div
                class={
                    classes!(
                        "h-100vh", "bg-black"
                    )
                }
            >
            </div>
            <div
                class={
                    classes!(
                        "absolute", "top-50",
                        "bg-white", "text-black", "text-5xl",
                    )
                }
            >
                { for children.iter() }
            </div>
        </>
    }
}
