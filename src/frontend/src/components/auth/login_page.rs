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
        <div class="min-h-screen grid grid-cols-1 lg:grid-cols-5">

            // ── Left: brand panel ────────────────────────────────────────
            <div class="hidden lg:flex lg:col-span-2 flex-col justify-between bg-slate-900 px-12 py-16">
                <div>
                    <span class="text-2xl font-bold tracking-tight text-white">{"Mosaic"}</span>
                </div>
                <div class="space-y-4">
                    <div class="w-8 h-1 bg-amber-500 rounded" />
                    <p class="text-slate-300 text-sm leading-relaxed max-w-xs">
                        {"Your personal data portal. Unified metrics, forms, and dashboards — one login, one place."}
                    </p>
                </div>
                <p class="text-slate-600 text-xs">{"Self-hosted · Open source"}</p>
            </div>

            // ── Right: form ──────────────────────────────────────────────
            <div class="lg:col-span-3 flex flex-col justify-center px-8 py-16 sm:px-16 bg-stone-50">
                <div class="w-full max-w-sm mx-auto">

                    // Mobile-only brand
                    <div class="lg:hidden mb-8">
                        <span class="text-xl font-bold text-slate-900">{"Mosaic"}</span>
                    </div>

                    <h1 class="text-2xl font-bold text-stone-900">{"Sign in"}</h1>
                    <p class="mt-1 text-sm text-stone-500 mb-8">{"Enter your credentials to continue"}</p>

                    if let Some(err) = (*error).clone() {
                        <div class="mb-4 rounded-md bg-red-50 border border-red-200 px-4 py-3 text-sm text-red-700">
                            { err }
                        </div>
                    }

                    <form onsubmit={on_submit} class="space-y-4">
                        <div class="space-y-1">
                            <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500"
                                   for="username">
                                {"Username"}
                            </label>
                            <input
                                id="username" type="text" autocomplete="username" required=true
                                disabled={*loading}
                                value={(*username).clone()}
                                oninput={on_username}
                                class="w-full rounded-md border border-stone-300 bg-white px-3 py-2.5 text-sm
                                       text-stone-900 placeholder-stone-400
                                       focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent
                                       disabled:opacity-50 transition"
                            />
                        </div>

                        <div class="space-y-1">
                            <label class="block text-xs font-semibold uppercase tracking-wider text-stone-500"
                                   for="password">
                                {"Password"}
                            </label>
                            <input
                                id="password" type="password" autocomplete="current-password" required=true
                                disabled={*loading}
                                value={(*password).clone()}
                                oninput={on_password}
                                class="w-full rounded-md border border-stone-300 bg-white px-3 py-2.5 text-sm
                                       text-stone-900 placeholder-stone-400
                                       focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent
                                       disabled:opacity-50 transition"
                            />
                        </div>

                        <button
                            type="submit"
                            disabled={*loading}
                            class="mt-2 w-full rounded-md bg-amber-500 px-4 py-2.5 text-sm font-semibold
                                   text-slate-900 hover:bg-amber-400 active:bg-amber-600
                                   disabled:opacity-50 transition-colors"
                        >
                            if *loading { {"Signing in…"} } else { {"Sign in"} }
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}