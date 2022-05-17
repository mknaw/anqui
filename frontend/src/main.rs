use crate::routes::*;
use yew::prelude::*;
use yew_router::prelude::*;

mod api;
mod cards;
mod decks;
mod login;
mod routes;
mod utils;

#[derive(PartialEq, Properties)]
pub struct LayoutProps {
    children: Children,
}

#[function_component(Layout)]
pub fn layout(LayoutProps { children }: &LayoutProps) -> Html {
    html! {
        <div class={ classes!("w-full", "h-screen", "max-h-screen", "flex", "flex-col", "overflow-hidden") }>
            <nav class={ classes!("w-full", "text-3xl", "flex", "justify-end", "p-3") }>
                <span class={ classes!("px-2") }>
                    <Link<AppRoute> to={ AppRoute::Decks }>{ "🏡" }</Link<AppRoute>>
                </span>
                <span class={ classes!("px-2") }>
                    <a href={ "/logout/" }>
                        { "👋" }
                    </a>
                </span>
            </nav>
            <div
                id={ "content" }
                class={ classes!("container", "mx-auto", "flex", "flex-col", "justify-center", "h-full") }
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
