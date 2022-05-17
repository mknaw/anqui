use yew::prelude::*;
use yew_router::prelude::*;

use crate::lib::*;
use crate::routes::*;

mod lib;
mod routes;
mod views;

#[derive(PartialEq, Properties)]
pub struct LayoutProps {
    children: Children,
}

#[function_component(Layout)]
pub fn layout(LayoutProps { children }: &LayoutProps) -> Html {
    html! {
        <div
            class={
                classes!(
                    "w-full", "h-screen", "max-h-screen",
                    "flex", "flex-col", "justify-between", "items-center",
                )
            }
        >
            <nav
                class={
                    classes!(
                        "absolute",  "right-0", "text-6xl", "lg:text-3xl", "flex", "justify-end", "p-5"
                    )
                }
        >
                <span class={ classes!("px-2") }>
                    <Link<AppRoute> to={ AppRoute::Decks }>{ emojis::HOME }</Link<AppRoute>>
                </span>
                <span class={ classes!("px-2") }>
                    <a href={ "/logout/" }>
                        { emojis::WAVE }
                    </a>
                </span>
            </nav>
            <div
                id={ "content" }
                class={ classes!("flex", "flex-col", "justify-center", "h-full", "w-full", "items-center") }
            >
                { for children.iter() }
            </div>
        </div>
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
