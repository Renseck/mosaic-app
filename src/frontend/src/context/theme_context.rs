// TODO: Complete in Phase 8
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeState {
    pub dark: bool,
}

/* ============================================================================================== */
#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub children: Children,
}

/* ============================================================================================== */
#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    html! {
        { for props.children.iter() }
    }
}