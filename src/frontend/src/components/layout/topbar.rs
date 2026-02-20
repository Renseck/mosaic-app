use yew::prelude::*;
use yew_router::{navigator, prelude::*};
use crate::api::auth;
use crate::context::auth_context::AuthAction;
use crate::hooks::use_auth::use_auth;
use crate::router::Route;

#[function_component(Topbar)]
pub fn topbar() -> Html {
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
        <header class="flex items-center justify-between h-12 px-6 bg-white border-b border-gray-200 shrink-0">
            /* Left side — breadcrumb placeholder */
            <div class="text-sm text-gray-500">{"Portal"}</div>

            /* Right side — user info + logout */
            <div class="flex items-center gap-4">
                if let Some(user) = &auth.user {
                    <span class="text-sm text-gray-700">
                        { &user.username }
                        <span class="ml-1 text-xs text-gray-400">
                            { format!("({})", user.role) }
                        </span>
                    </span>
                    <button
                        onclick={on_logout}
                        class="text-sm text-red-600 hover:text-red-800 transition-colors"
                    >
                        {"Sign out"}
                    </button>
                }
            </div>
        </header>
    }
}