use web_sys::Element;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::lib::*;
use crate::routes::*;

mod components;
mod lib;
mod routes;
mod views;

const OVERLAY_ID: &str = "overlay";

#[derive(Clone, Debug, PartialEq)]
pub struct AppContext {
    set_title: Callback<String>,
    set_modal: Callback<Option<Html>>,
}

#[derive(PartialEq, Properties)]
pub struct LayoutProps {
    children: Children,
}

#[function_component(Layout)]
pub fn layout(LayoutProps { children }: &LayoutProps) -> Html {
    let title = use_state(|| "".to_string());
    let set_title = {
        let title = title.clone();
        Callback::from(move |new_title: String| title.set(new_title))
    };

    let modal = use_state(|| None);
    let set_modal = {
        let modal = modal.clone();
        Callback::from(move |new_modal: Option<Html>| modal.set(new_modal))
    };
    let on_overlay_click = {
        let modal = modal.clone();
        Callback::from(move |e: MouseEvent| {
            let target: Element = e.target_unchecked_into();
            if target.id() == OVERLAY_ID {
                // Wonder if better not just to hide it so the same one could be pulled up again.
                // Probably not hugely consequential one way or another though.
                modal.set(None);
            }
        })
    };

    let ctx = use_state(|| AppContext {
        set_title,
        set_modal,
    });

    html! {
        <ContextProvider<AppContext> context={ (*ctx).clone() }>
            <>
                <div
                    class={
                        classes!(
                            "w-full", "h-screen", "max-h-screen", "overflow-hidden",
                            "flex", "flex-col", "items-center",
                        )
                    }
                >
                    <nav
                        class={
                            classes!(
                                "bg-blk", "h-[5vh]", "w-full", "z-10",
                                "portrait:text-6xl", "lg:text-3xl", "flex", "justify-center", "items-center",
                            )
                        }
                    >
                        <span>{ (*title).clone() }</span>
                        <span class={ classes!("absolute", "right-5") }>
                            <span class={ classes!("px-2") }>
                                <Link<AppRoute> to={ AppRoute::Decks }>
                                    { emojis::HOME }
                                </Link<AppRoute>>
                            </span>
                            <span class={ classes!("px-2") }>
                                <a href={ "/logout/" }>
                                    { emojis::WAVE }
                                </a>
                            </span>
                        </span>
                    </nav>
                    <div
                        id={ "content" }
                        class={
                            classes!(
                                "flex", "flex-col", "justify-center", "h-95vh", "w-full", "items-center",
                            )
                        }
                    >
                        { for children.iter() }
                    </div>
                </div>
                {
                    if let Some(modal) = (*modal).clone() {
                        // TODO && modal shown?
                        // or could initialize with an empty shown=false modal instead of `Option`
                        html! {
                            <>
                                <div
                                    id={ OVERLAY_ID }
                                    onclick={ on_overlay_click }
                                    class={
                                        classes!(
                                            "absolute", "w-screen", "h-screen", "top-0", "left-0", "z-30",
                                            "flex", "justify-center", "items-center",
                                            "bg-blur", "bg-black/50",
                                        )
                                    }
                                >
                                    <div>
                                        { modal }
                                    </div>
                                </div>
                            </>
                        }
                    } else {
                        html! {}
                    }
                }

            </>
        </ContextProvider<AppContext>>
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<MainRoute> render={Switch::render(main_switch)} />
        </BrowserRouter>
    }
}

fn main() {
    // TODO handle unauthed API requests (redirect to login for example)
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Main>();
}
