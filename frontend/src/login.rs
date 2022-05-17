use serde_json::json;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::post_vanilla;
use crate::routes::AppRoute;

#[function_component(Login)]
pub fn login() -> Html {
    let history = use_history().unwrap();
    let username_node_ref = use_node_ref();
    let password_node_ref = use_node_ref();
    let button_node_ref = use_node_ref();
    let error = use_state(|| "".to_string());

    let onsubmit = {
        let username_node_ref = username_node_ref.clone();
        let password_node_ref = password_node_ref.clone();
        let button_node_ref = button_node_ref.clone();
        let error = error.clone();

        Callback::from(move |e: FocusEvent| {
            e.prevent_default();

            let username = username_node_ref.cast::<HtmlInputElement>();
            let password = password_node_ref.cast::<HtmlInputElement>();
            let button = button_node_ref.cast::<Element>().unwrap();
            button.set_class_name("text-gray-600 pointer-events-none");

            let history = history.clone();
            let error = error.clone();

            if let (Some(username), Some(password)) = (username, password) {
                let username = username.value();
                let password = password.value();
                if username.is_empty() || password.is_empty() {
                    // TODO communicate to user
                    log::info!("its empty");
                    // TODO probably must be a better way to "clean up after any fuckup"
                    button.set_class_name("");
                    return;
                }
                let payload = json!({
                    "username": username,
                    "password": password,
                });
                wasm_bindgen_futures::spawn_local(async move {
                    match post_vanilla("/login/", payload).await {
                        Ok(_) => history.push(AppRoute::Decks),
                        Err(e) => {
                            button.set_class_name("");
                            error.set(e.to_string());
                        }
                    }
                });
            };
        })
    };

    html! {
        <div
            class={
                classes!(
                    "h-screen", "flex", "flex-col", "justify-center", "items-center",
                    "text-7xl",
                    "lg:text-3xl",
                )
            }
        >
            <form { onsubmit } class={ classes!("mb-32") }>
                // TODO would be nice if this appeared in a "fixed" place, without offsetting inputs.
                <div hidden={ (*error).is_empty() } class={ classes!("py-2") }>
                    { (*error).clone() }
                </div>
                <div class={ classes!("py-2") }>
                    <input
                        type="text"
                        name="username"
                        placeholder="Nom d'utilisateur"
                        ref={ username_node_ref }
                    />
                </div>
                <div class={ classes!("py-2") }>
                    <input
                        type="password"
                        name="password"
                        placeholder="Mot de passe"
                        ref={ password_node_ref }
                    />
                </div>
                <div class={ classes!("py-2") }>
                    <button
                        type={ "submit" }
                        ref={ button_node_ref }
                    >
                        { "Se connecter" }
                    </button>
                </div>
            </form>
        </div>
    }
}
