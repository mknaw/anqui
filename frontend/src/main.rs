use crate::routes::{switch, Route};
use yew::prelude::*;
use yew_router::prelude::*;

mod api;
mod cards;
mod decks;
mod login;
mod routes;

#[function_component(Main)]
fn app() -> Html {
    html! {
        <div id={ "content" }>
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </div>
    }
}

fn main() {
    // TODO handle unauthed API requests (redirect to login for example)
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Some info");
    yew::start_app::<Main>();
}
