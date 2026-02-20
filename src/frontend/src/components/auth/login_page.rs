use yew::prelude::*;
use yew_router::prelude::*;
use crate::api::auth;
use crate::context::auth_context::AuthAction;
use crate::hooks::use_auth::use_auth;
use crate::router::Route;

#[function_component(LoginPage)]
pub fn login_page() -> Html {
    let auth = use_auth();
    let navigator = use_navigator().expect("navigator not found");

    let username = use_state(String::new);
    let password = use_state(String::new);
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);

    // If already logged in, redirect away
    if auth.user.is_some() {
        navigator.push(&Route::DashboardList);
    }

    let on_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let on_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input : web_sys::HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let on_submit = {
        let auth = auth.clone();
        let navigator = navigator.clone();
        let username = username.clone();
        let password = password.clone();
        let error = error.clone();
        let loading = loading.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let auth = auth.clone();
            let navigator = navigator.clone();
            let u = (*username).clone();
            let p = (*password).clone();
            let error = error.clone();
            let loading = loading.clone();

            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match auth::login(&u, &p).await {
                    Ok(user) => {
                        auth.dispatch(AuthAction::SetUser(user));
                        navigator.push(&Route::DashboardList);
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <div class="w-full max-w-sm space-y-6">
                <div class="text-center">
                    <h1 class="text-3xl font-bold text-gray-900">{"Mosaic"}</h1>
                    <p class="mt-1 text-sm text-gray-500">{"Sign in to your portal"}</p>
                </div>

                <form onsubmit={on_submit} class="bg-white shadow rounded-lg px-8 py-8 space-y-4">
                    if let Some(err) = (*error).clone() {
                        <div class="rounded bg-red-50 border border-red-200 px-4 py-2 text-sm text-red-700">
                            { err }
                        </div>
                    }

                    <div class="space-y-1">
                        <label class="block text-sm font-medium text-gray-700" for="username">
                            {"Username"}
                        </label>
                        <input
                            id="username"
                            type="text"
                            autocomplete="username"
                            required=true
                            disabled={*loading}
                            value={(*username).clone()}
                            oninput={on_username}
                            class="w-full rounded border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                        />
                    </div>

                    <div class="space-y-1">
                        <label class="block text-sm font-medium text-gray-700" for="password">
                            {"Password"}
                        </label>
                        <input
                            id="password"
                            type="password"
                            autocomplete="current-password"
                            required=true
                            disabled={*loading}
                            value={(*password).clone()}
                            oninput={on_password}
                            class="w-full rounded border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
                        />
                    </div>

                    <button
                        type="submit"
                        disabled={*loading}
                        class="w-full rounded bg-indigo-600 px-4 py-2 text-sm font-medium text-white hover:bg-indigo-700 disabled:opacity-50 transition-colors"
                    >
                        if *loading { {"Signing in…"} } else { {"Sign in"} }
                    </button>
                </form>
            </div>
        </div>
    }
}