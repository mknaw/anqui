use yew::prelude::*;
use yew_router::prelude::*;

use crate::lib::*;
use crate::routes::*;

mod lib;
mod routes;
mod views;

#[derive(Clone, Debug, PartialEq)]
pub struct AppContext {
    set_title: Callback<String>,
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

    let ctx = use_state(|| AppContext { set_title });

    html! {
        <ContextProvider<AppContext> context={(*ctx).clone()}>
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
