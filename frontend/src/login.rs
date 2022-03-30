use yew::prelude::*;

#[function_component(Login)]
pub fn login() -> Html {
    let username_node_ref = use_node_ref();
    let password_node_ref = use_node_ref();

    html! {
        <form action="" method="post">
            <div>
                <label>{ "Username" }
                    <input
                        type="text"
                        name="username"
                        ref={ username_node_ref }
                    />
                </label>
            </div>
            <div>
                <label>{ "Password" }
                    <input
                        type="password"
                        name="password"
                        ref={ password_node_ref }
                    />
                </label>
            </div>
            <div>
                <button>{ "Log in" }</button>
            </div>
        </form>
    }
}
