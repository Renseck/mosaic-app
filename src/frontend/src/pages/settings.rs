use yew::prelude::*;
use crate::api::auth;
use crate::components::common::{use_toast, ToastKind};
use crate::context::theme_context::{ThemeAction, ThemeContext};

#[function_component(SettingsPage)]
pub fn settings_page() -> Html {
    let theme      = use_context::<ThemeContext>().expect("ThemeContext missing");
    let show_toast = use_toast();

    // ── Change-password form state ──────────────────────────────────────────
    let current_pw = use_state(String::new);
    let new_pw     = use_state(String::new);
    let confirm_pw = use_state(String::new);
    let submitting = use_state(|| false);
    let pw_error   = use_state(|| None::<String>);

    let on_submit_pw = {
        let current_pw = current_pw.clone();
        let new_pw     = new_pw.clone();
        let confirm_pw = confirm_pw.clone();
        let submitting = submitting.clone();
        let pw_error   = pw_error.clone();
        let show_toast = show_toast.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let cur = (*current_pw).clone();
            let new = (*new_pw).clone();
            let con = (*confirm_pw).clone();

            if new != con {
                pw_error.set(Some("New passwords do not match".to_string()));
                return;
            }
            if new.len() < 8 {
                pw_error.set(Some("Password must be at least 8 characters".to_string()));
                return;
            }

            pw_error.set(None);
            submitting.set(true);

            let current_pw = current_pw.clone();
            let new_pw     = new_pw.clone();
            let confirm_pw = confirm_pw.clone();
            let submitting = submitting.clone();
            let pw_error   = pw_error.clone();
            let show_toast = show_toast.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match auth::change_password(&cur, &new).await {
                    Ok(_) => {
                        current_pw.set(String::new());
                        new_pw.set(String::new());
                        confirm_pw.set(String::new());
                        show_toast.emit(("Password updated".to_string(), ToastKind::Success));
                    }
                    Err(e) => {
                        pw_error.set(Some(e.to_string()));
                        show_toast.emit(("Failed to update password".to_string(), ToastKind::Error));
                    }
                }
                submitting.set(false);
            });
        })
    };

    let input_class = "w-full rounded border border-stone-300 dark:border-stone-600 \
                       bg-white dark:bg-stone-700 text-stone-800 dark:text-stone-100 \
                       text-sm px-3 py-2 focus:outline-none focus:ring-2 focus:ring-amber-400";

    html! {
        <div class="max-w-2xl space-y-8">
            <h1 class="text-xl font-semibold text-stone-800 dark:text-stone-100">{ "Settings" }</h1>

            // ── Appearance ──────────────────────────────────────────────────────
            <section class="bg-white dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 p-6">
                <h2 class="text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400 mb-4">
                    { "Appearance" }
                </h2>
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm font-medium text-stone-800 dark:text-stone-100">{ "Dark mode" }</p>
                        <p class="text-xs text-stone-400 mt-0.5">{ "Toggle dark theme" }</p>
                    </div>
                    // Toggle pill
                    <button
                        onclick={{
                            let theme = theme.clone();
                            Callback::from(move |_: MouseEvent| theme.dispatch(ThemeAction::Toggle))
                        }}
                        aria-label="Toggle dark mode"
                        class={format!(
                            "relative inline-flex h-6 w-11 items-center rounded-full \
                             transition-colors {}",
                            if theme.dark { "bg-amber-500" } else { "bg-stone-300 dark:bg-stone-600" }
                        )}
                    >
                        <span class={format!(
                            "inline-block h-4 w-4 transform rounded-full bg-white shadow \
                             transition-transform {}",
                            if theme.dark { "translate-x-6" } else { "translate-x-1" }
                        )} />
                    </button>
                </div>
            </section>

            // ── Change password ─────────────────────────────────────────────────
            <section class="bg-white dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 p-6">
                <h2 class="text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400 mb-4">
                    { "Change Password" }
                </h2>
                <form onsubmit={on_submit_pw} class="space-y-4 max-w-sm">
                    <div>
                        <label class="block text-xs font-medium text-stone-600 dark:text-stone-300 mb-1">
                            { "Current password" }
                        </label>
                        <input
                            type="password" required=true
                            value={(*current_pw).clone()}
                            oninput={{
                                let s = current_pw.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    s.set(v.value());
                                })
                            }}
                            class={input_class}
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-stone-600 dark:text-stone-300 mb-1">
                            { "New password" }
                        </label>
                        <input
                            type="password" required=true
                            value={(*new_pw).clone()}
                            oninput={{
                                let s = new_pw.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    s.set(v.value());
                                })
                            }}
                            class={input_class}
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-stone-600 dark:text-stone-300 mb-1">
                            { "Confirm new password" }
                        </label>
                        <input
                            type="password" required=true
                            value={(*confirm_pw).clone()}
                            oninput={{
                                let s = confirm_pw.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    s.set(v.value());
                                })
                            }}
                            class={input_class}
                        />
                    </div>
                    if let Some(err) = &*pw_error {
                        <p class="text-xs text-red-500">{ err }</p>
                    }
                    <button
                        type="submit"
                        disabled={*submitting}
                        class="px-4 py-2 text-sm font-medium rounded bg-amber-500 text-stone-900 \
                               hover:bg-amber-400 disabled:opacity-50 transition-colors"
                    >
                        { if *submitting { "Updating…" } else { "Update password" } }
                    </button>
                </form>
            </section>
        </div>
    }
}
