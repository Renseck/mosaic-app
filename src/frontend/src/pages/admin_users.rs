use yew::prelude::*;
use crate::api::users;
use crate::components::common::{use_toast, ToastKind};
use crate::models::User;

/* ============================================================================================== */
#[function_component(AdminUsersPage)]
pub fn admin_users_page() -> Html {
    let user_list  = use_state(Vec::<User>::new);
    let loading    = use_state(|| true);
    let error      = use_state(|| None::<String>);
    let show_toast = use_toast();

    // ── Create-user form state ──────────────────────────────────────────────
    let new_username  = use_state(String::new);
    let new_password  = use_state(String::new);
    let new_email     = use_state(String::new);
    let creating      = use_state(|| false);
    let create_error  = use_state(|| None::<String>);

    // ── Fetch users on mount ────────────────────────────────────────────────
    {
        let user_list = user_list.clone();
        let loading   = loading.clone();
        let error     = error.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match users::list_users().await {
                    Ok(list) => { user_list.set(list); loading.set(false); }
                    Err(e)   => { error.set(Some(e.to_string())); loading.set(false); }
                }
            });
            || ()
        });
    }

    // ── Role change callback ────────────────────────────────────────────────
    let on_role_change = {
        let user_list  = user_list.clone();
        let show_toast = show_toast.clone();
        Callback::from(move |(id, role): (String, String)| {
            let user_list  = user_list.clone();
            let show_toast = show_toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match users::update_user_role(&id, &role).await {
                    Ok(updated) => {
                        user_list.set(
                            (*user_list).iter().map(|u| {
                                if u.id == updated.id { updated.clone() } else { u.clone() }
                            }).collect()
                        );
                        show_toast.emit(("Role updated".to_string(), ToastKind::Success));
                    }
                    Err(e) => {
                        show_toast.emit((format!("Failed: {e}"), ToastKind::Error));
                    }
                }
            });
        })
    };

    // ── Password reset callback ───────────────────────────────────────────
    let on_reset_password = {
        let show_toast = show_toast.clone();
        Callback::from(move |id: String| {
            let show_toast = show_toast.clone();
            let temp_pw = generate_temp_password(12);
            let temp_pw_display = temp_pw.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match users::reset_user_password(&id, &temp_pw).await {
                    Ok(_) => {
                        show_toast.emit((
                            format!("Password reset to: {temp_pw_display}"),
                            ToastKind::Success,
                        ));
                    }
                    Err(e) => {
                        show_toast.emit((format!("Failed: {e}"), ToastKind::Error));
                    }
                }
            });
        })
    };

    // ── Create user callback ────────────────────────────────────────────────
    let on_create = {
        let new_username = new_username.clone();
        let new_password = new_password.clone();
        let new_email    = new_email.clone();
        let creating     = creating.clone();
        let create_error = create_error.clone();
        let user_list    = user_list.clone();
        let show_toast   = show_toast.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username  = (*new_username).clone();
            let password  = (*new_password).clone();
            let email_str = (*new_email).clone();
            let email: Option<String> = if email_str.trim().is_empty() {
                None
            } else {
                Some(email_str.trim().to_string())
            };

            create_error.set(None);
            creating.set(true);

            let new_username = new_username.clone();
            let new_password = new_password.clone();
            let new_email    = new_email.clone();
            let creating     = creating.clone();
            let create_error = create_error.clone();
            let user_list    = user_list.clone();
            let show_toast   = show_toast.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match users::create_user(&username, &password, email.as_deref()).await {
                    Ok(new_user) => {
                        let mut list = (*user_list).clone();
                        list.push(new_user);
                        user_list.set(list);
                        new_username.set(String::new());
                        new_password.set(String::new());
                        new_email.set(String::new());
                        show_toast.emit(("User created".to_string(), ToastKind::Success));
                    }
                    Err(e) => {
                        create_error.set(Some(e.to_string()));
                        show_toast.emit((format!("Failed: {e}"), ToastKind::Error));
                    }
                }
                creating.set(false);
            });
        })
    };

    let input_class = "w-full rounded border border-stone-300 dark:border-stone-600 \
                       bg-white dark:bg-stone-700 text-stone-800 dark:text-stone-100 \
                       text-sm px-3 py-2 focus:outline-none focus:ring-2 focus:ring-amber-400";

    html! {
        <div class="space-y-8">
            <h1 class="text-xl font-semibold text-stone-800 dark:text-stone-100">{ "Users" }</h1>

            // ── User table ──────────────────────────────────────────────────────
            <section class="bg-white dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 overflow-hidden">
                if *loading {
                    <div class="p-8 flex justify-center">
                        <div class="w-6 h-6 border-2 border-amber-500 border-t-transparent rounded-full animate-spin" />
                    </div>
                } else if let Some(err) = &*error {
                    <p class="p-6 text-sm text-red-500">{ err }</p>
                } else {
                    <table class="w-full text-sm">
                        <thead class="border-b border-stone-200 dark:border-stone-700">
                            <tr class="text-left text-xs font-semibold uppercase tracking-wider \
                                       text-stone-500 dark:text-stone-400">
                                <th class="px-6 py-3">{ "Username" }</th>
                                <th class="px-6 py-3">{ "Email" }</th>
                                <th class="px-6 py-3">{ "Role" }</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-stone-100 dark:divide-stone-700">
                            { for (*user_list).iter().map(|u| {
                                let on_role = {
                                    let on_role_change = on_role_change.clone();
                                    let id = u.id.clone();
                                    Callback::from(move |role: String| {
                                        on_role_change.emit((id.clone(), role));
                                    })
                                };
                                let on_reset = {
                                    let on_reset_password = on_reset_password.clone();
                                    let id = u.id.clone();
                                    Callback::from(move |_: ()| {
                                        on_reset_password.emit(id.clone());
                                    })
                                };
                                html! {
                                    <UserRow key={u.id.clone()} user={u.clone()} on_role_change={on_role} on_reset_password={on_reset} />
                                }
                            })}
                        </tbody>
                    </table>
                }
            </section>

            // ── Create new user ─────────────────────────────────────────────────
            <section class="bg-white dark:bg-stone-800 rounded-lg border border-stone-200 dark:border-stone-700 p-6">
                <h2 class="text-xs font-semibold uppercase tracking-wider text-stone-500 dark:text-stone-400 mb-4">
                    { "Create New User" }
                </h2>
                <form onsubmit={on_create} class="grid grid-cols-1 sm:grid-cols-3 gap-4 max-w-2xl">
                    <div>
                        <label class="block text-xs font-medium text-stone-600 dark:text-stone-300 mb-1">
                            { "Username *" }
                        </label>
                        <input
                            type="text" required=true
                            value={(*new_username).clone()}
                            oninput={{
                                let s = new_username.clone();
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
                            { "Password *" }
                        </label>
                        <input
                            type="password" required=true
                            value={(*new_password).clone()}
                            oninput={{
                                let s = new_password.clone();
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
                            { "Email (optional)" }
                        </label>
                        <input
                            type="email"
                            value={(*new_email).clone()}
                            oninput={{
                                let s = new_email.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    s.set(v.value());
                                })
                            }}
                            class={input_class}
                        />
                    </div>
                    if let Some(err) = &*create_error {
                        <p class="sm:col-span-3 text-xs text-red-500">{ err }</p>
                    }
                    <div class="sm:col-span-3">
                        <button
                            type="submit"
                            disabled={*creating}
                            class="px-4 py-2 text-sm font-medium rounded bg-amber-500 text-stone-900 \
                                   hover:bg-amber-400 disabled:opacity-50 transition-colors"
                        >
                            { if *creating { "Creating…" } else { "Create user" } }
                        </button>
                    </div>
                </form>
            </section>
        </div>
    }
}

/* ============================================================================================== */
/*                                          UserRow sub-component                                */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
struct UserRowProps {
    user: User,
    on_role_change: Callback<String>,
    on_reset_password: Callback<()>,
}

#[function_component(UserRow)]
fn user_row(props: &UserRowProps) -> Html {
    let confirming = use_state(|| false);

    let on_change = {
        let cb = props.on_role_change.clone();
        Callback::from(move |e: web_sys::Event| {
            let target: web_sys::HtmlSelectElement = e.target_unchecked_into();
            cb.emit(target.value());
        })
    };

    let on_reset_click = {
        let confirming = confirming.clone();
        Callback::from(move |_: MouseEvent| {
            confirming.set(true);
        })
    };

    let on_confirm = {
        let confirming = confirming.clone();
        let cb = props.on_reset_password.clone();
        Callback::from(move |_: MouseEvent| {
            confirming.set(false);
            cb.emit(());
        })
    };

    let on_cancel = {
        let confirming = confirming.clone();
        Callback::from(move |_: MouseEvent| {
            confirming.set(false);
        })
    };

    html! {
        <tr class="text-stone-700 dark:text-stone-300">
            <td class="px-6 py-3 font-medium">{ &props.user.username }</td>
            <td class="px-6 py-3 text-stone-500 dark:text-stone-400">
                { props.user.email.as_deref().unwrap_or("—") }
            </td>
            <td class="px-6 py-3">
                <select
                    onchange={on_change}
                    class="rounded border border-stone-200 dark:border-stone-600 bg-white \
                           dark:bg-stone-700 text-stone-700 dark:text-stone-200 text-xs \
                           px-2 py-1 focus:outline-none focus:ring-1 focus:ring-amber-400"
                >
                    <option value="admin"  selected={props.user.role.to_string() == "admin"}>  { "Admin"  } </option>
                    <option value="editor" selected={props.user.role.to_string() == "editor"}> { "Editor" } </option>
                    <option value="viewer" selected={props.user.role.to_string() == "viewer"}> { "Viewer" } </option>
                </select>
            </td>
            <td class="px-6 py-3">
                if *confirming {
                    <span class="inline-flex items-center gap-2">
                        <span class="text-xs text-red-500">{"Sure?"}</span>
                        <button onclick={on_confirm}
                            class="text-xs font-medium text-red-600 hover:text-red-500 transition-colors">
                            {"Yes"}
                        </button>
                        <button onclick={on_cancel}
                            class="text-xs text-stone-400 hover:text-stone-600 dark:hover:text-stone-200 transition-colors">
                            {"No"}
                        </button>
                    </span>
                } else {
                    <button onclick={on_reset_click}
                        title="Reset password to Welcome1234!"
                        class="text-xs text-stone-400 hover:text-amber-600 dark:hover:text-amber-300 transition-colors">
                        {"Reset pw"}
                    </button>
                }
            </td>
        </tr>
    }
}

/* ============================================================================================== */
/*                                             Helpers                                            */
/* ============================================================================================== */

fn generate_temp_password(len: usize) -> String {
    let charset = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789!@#$";
    let mut buf = vec![0u8; len];
    getrandom::fill(&mut buf).expect("getrandom failed");
    buf.iter()
        .map(|b| charset[*b as usize % charset.len()] as char)
        .collect()
}