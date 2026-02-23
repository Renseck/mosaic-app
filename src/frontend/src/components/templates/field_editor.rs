use yew::prelude::*;
use crate::models::template::FieldDefinition;

#[derive(Properties, PartialEq)]
pub struct FieldEditorProps {
    pub fields:    Vec<FieldDefinition>,
    pub on_change: Callback<Vec<FieldDefinition>>,
}

#[function_component(FieldEditor)]
pub fn field_editor(props: &FieldEditorProps) -> Html {
    let new_name      = use_state(String::new);
    let new_type      = use_state(|| "number".to_string());
    let new_unit      = use_state(String::new);
    let name_error    = use_state(|| Option::<String>::None);

    let on_add = {
        let fields     = props.fields.clone();
        let on_change  = props.on_change.clone();
        let new_name   = new_name.clone();
        let new_type   = new_type.clone();
        let new_unit   = new_unit.clone();
        let name_error = name_error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let name = (*new_name).trim().to_lowercase();

            // Validate: alphanumeric + underscore, must start with letter
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                || name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true)
            {
                name_error.set(Some(
                    "Name must be lowercase letters, numbers, and underscores (start with a letter)".into()
                ));
                return;
            }
            if fields.iter().any(|f| f.name == name) {
                name_error.set(Some("A field with this name already exists".into()));
                return;
            }
            name_error.set(None);

            let unit = (*new_unit).trim().to_string();
            let mut updated = fields.clone();
            updated.push(FieldDefinition {
                name,
                field_type: (*new_type).clone(),
                unit: if unit.is_empty() { None } else { Some(unit) },
            });
            on_change.emit(updated);
            new_name.set(String::new());
            new_unit.set(String::new());
        })
    };

    let on_remove = {
        let fields    = props.fields.clone();
        let on_change = props.on_change.clone();
        move |idx: usize| {
            let on_change = on_change.clone();
            let mut updated = fields.clone();
            updated.remove(idx);
            Callback::from(move |_: MouseEvent| on_change.emit(updated.clone()))
        }
    };

    html! {
        <div class="space-y-3">
            // ── Existing fields ─────────────────────────────────────────────
            if !props.fields.is_empty() {
                <div class="space-y-1.5">
                    { for props.fields.iter().enumerate().map(|(i, field)| {
                        let remove_cb = on_remove(i);
                        let type_badge_class = match field.field_type.as_str() {
                            "number" => "bg-amber-100 text-amber-800",
                            "date"   => "bg-blue-100 text-blue-800",
                            "select" => "bg-purple-100 text-purple-800",
                            _        => "bg-stone-100 text-stone-700",
                        };
                        html! {
                            <div class="flex items-center gap-2 px-3 py-2 bg-white border border-stone-200 rounded-lg">
                                <span class="font-mono text-sm font-medium text-stone-800 flex-1">
                                    { &field.name }
                                    if let Some(ref unit) = field.unit {
                                        <span class="ml-1 text-xs text-stone-400">{ format!("({unit})") }</span>
                                    }
                                </span>
                                <span class={classes!(
                                    "text-xs", "font-semibold", "px-2", "py-0.5", "rounded-full",
                                    type_badge_class
                                )}>
                                    { &field.field_type }
                                </span>
                                <button
                                    type="button"
                                    onclick={remove_cb}
                                    class="text-stone-300 hover:text-red-500 transition-colors text-sm ml-1"
                                    title="Remove field"
                                >
                                    {"✕"}
                                </button>
                            </div>
                        }
                    })}
                </div>
            }

            // ── Add new field ───────────────────────────────────────────────
            <form onsubmit={on_add}
                class="flex flex-wrap gap-2 p-3 bg-stone-50 border border-dashed border-stone-300 rounded-lg">

                if let Some(err) = (*name_error).clone() {
                    <p class="w-full text-xs text-red-600">{ err }</p>
                }

                <input
                    type="text" placeholder="field_name" required=true
                    value={(*new_name).clone()}
                    oninput={Callback::from({
                        let new_name = new_name.clone();
                        move |e: InputEvent| {
                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                            new_name.set(el.value());
                        }
                    })}
                    class="flex-1 min-w-24 rounded border border-stone-300 px-2 py-1.5 text-sm font-mono
                           focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent"
                />

                <select
                    value={(*new_type).clone()}
                    onchange={Callback::from({
                        let new_type = new_type.clone();
                        move |e: Event| {
                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                            new_type.set(el.value());
                        }
                    })}
                    class="rounded border border-stone-300 px-2 py-1.5 text-sm bg-white
                           focus:outline-none focus:ring-2 focus:ring-amber-500"
                >
                    <option value="number">{"number"}</option>
                    <option value="text">{"text"}</option>
                    <option value="date">{"date"}</option>
                    <option value="select">{"select"}</option>
                </select>

                <input
                    type="text" placeholder="unit (optional)"
                    value={(*new_unit).clone()}
                    oninput={Callback::from({
                        let new_unit = new_unit.clone();
                        move |e: InputEvent| {
                            let el: web_sys::HtmlInputElement = e.target_unchecked_into();
                            new_unit.set(el.value());
                        }
                    })}
                    class="w-28 rounded border border-stone-300 px-2 py-1.5 text-sm
                           focus:outline-none focus:ring-2 focus:ring-amber-500 focus:border-transparent"
                />

                <button
                    type="submit"
                    class="px-3 py-1.5 text-sm font-semibold rounded bg-amber-500 text-slate-900
                           hover:bg-amber-400 transition-colors whitespace-nowrap"
                >
                    {"+ Add Field"}
                </button>
            </form>
        </div>
    }
}
