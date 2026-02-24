use yew::prelude::*;
use yew_router::prelude::*;
use crate::api::auth;
use crate::context::auth_context::AuthAction;
use crate::hooks::use_auth::use_auth;
use crate::router::Route;

#[derive(Properties, PartialEq)]
pub struct TopbarProps {
    /// Optional right-side slot for page-specific actions (edit toggle, add panel, etc.)
    #[prop_or_default]
    pub actions: Html,
    pub on_menu_toggle: Callback<()>,
}

#[function_component(Topbar)]
pub fn topbar(props: &TopbarProps) -> Html {
    let auth = use_auth();
    let navigator = use_navigator().expect("navigator not found");
    
    let on_logout = {
        let auth = auth.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            let auth = auth.clone();
            let navigator = navigator.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = auth::logout().await;
                auth.dispatch(AuthAction::ClearUser);
                navigator.push(&Route::Login);
            });
        })
    };

    html! {
        <header class="flex items-center justify-between h-12 px-4 bg-white dark:bg-stone-900 \
                        border-b border-gray-200 dark:border-stone-700 shrink-0">
            <div class="flex items-center gap-3">
                // Hamburger — visible on mobile only
                <button
                    onclick={props.on_menu_toggle.reform(|_: MouseEvent| ())}
                    class="md:hidden p-1.5 rounded text-stone-500 dark:text-stone-400 \
                           hover:bg-stone-100 dark:hover:bg-stone-800 transition-colors"
                    aria-label="Toggle sidebar"
                >
                    { "☰" }
                </button>
                { props.actions.clone() }
            </div>

            <div class="flex items-center gap-4">
                if let Some(user) = &auth.user {
                    <div class="flex items-center gap-2">
                        <div class="w-7 h-7 rounded-full bg-amber-500 flex items-center justify-center">
                            <span class="text-xs font-bold text-slate-900">
                                { user.username.chars().next().unwrap_or('?').to_uppercase().to_string() }
                            </span>
                        </div>
                        <div class="hidden sm:block text-right">
                            <p class="text-xs font-semibold text-stone-800 dark:text-stone-100 leading-none">
                                { &user.username }
                            </p>
                            <p class="text-xs text-stone-400 leading-none mt-0.5">
                                { user.role.to_string() }
                            </p>
                        </div>
                    </div>
                    <button
                        onclick={on_logout}
                        class="text-xs text-stone-400 hover:text-red-500 transition-colors"
                    >
                        { "Sign out" }
                    </button>
                }
            </div>
        </header>
    }
}