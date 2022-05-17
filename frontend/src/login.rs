use serde_json::json;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::post_vanilla;
use crate::routes::Route;

#[function_component(Login)]
pub fn login() -> Html {
    let history = use_history().unwrap();
    let username_node_ref = use_node_ref();
    let password_node_ref = use_node_ref();
    let error = use_state(|| "".to_string());

    let onsubmit = {
        let username_node_ref = username_node_ref.clone();
        let password_node_ref = password_node_ref.clone();
        let error = error.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();

            let username = username_node_ref.cast::<HtmlInputElement>();
            let password = password_node_ref.cast::<HtmlInputElement>();
            let history = history.clone();
            let error = error.clone();

            if let (Some(username), Some(password)) = (username, password) {
                let username = username.value();
                let password = password.value();
                if username.is_empty() || password.is_empty() {
                    // TODO some sort of warning? maybe not needed
                    log::info!("its empty");
                    return;
                }
                let payload = json!({
                    "username": username,
                    "password": password,
                });
                wasm_bindgen_futures::spawn_local(async move {
                    match post_vanilla("/login/", payload).await {
                        Ok(_) => history.push(Route::Decks),
                        Err(e) => error.set(e.to_string()),
                    }
                });
            };
        })
    };

    html! {
        <form { onsubmit }>
            {
                if (*error).is_empty() {
                    html! {}
                } else {
                    html! { <div>{ (*error).clone() }</div> }
                }
            }
            <div>
                <input
                    type="text"
                    name="username"
                    placeholder="Username"
                    ref={ username_node_ref }
                />
            </div>
            <div>
                <input
                    type="password"
                    name="password"
                    placeholder="Password"
                    ref={ password_node_ref }
                />
            </div>
            <div>
                <button type={ "submit" }>{ "Log in" }</button>
            </div>
        </form>
    }
}
