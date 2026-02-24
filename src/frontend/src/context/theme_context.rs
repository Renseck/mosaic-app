use std::rc::Rc;
use yew::prelude::*;
use gloo_storage::{LocalStorage, Storage};

/* ============================================================================================== */
/*                                              State                                            */
/* ============================================================================================== */

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeState {
    pub dark: bool,
}

pub enum ThemeAction {
    Toggle,
    SetDark(bool),
}

impl Reducible for ThemeState {
    type Action = ThemeAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let dark = match action {
            ThemeAction::Toggle     => !self.dark,
            ThemeAction::SetDark(v) => v,
        };
        Rc::new(ThemeState { dark })
    }
}

/* ============================================================================================== */
pub type ThemeContext = UseReducerHandle<ThemeState>;

/* ============================================================================================== */
#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let initial_dark: bool = LocalStorage::get("mosaic_dark").unwrap_or(false);
    let state = use_reducer(move || ThemeState { dark: initial_dark });

    // Sync `dark` class on <html> and persist to localStorage whenever the value changes.
    let dark = state.dark;
    use_effect_with(dark, |dark| {
        let dark = *dark;
        let doc = gloo_utils::document();
        let root = doc.document_element().expect("<html> element must exist");
        let cl = root.class_list();

        if dark {
            let _ = cl.add_1("dark");
        } else {
            let _ = cl.remove_1("dark");
        }

        let _ = LocalStorage::set("mosaic_dark", dark);
        || ()
    });

    html! {
        <ContextProvider<ThemeContext> context={state}>
            { for props.children.iter() }
        </ContextProvider<ThemeContext>>
    }
}
