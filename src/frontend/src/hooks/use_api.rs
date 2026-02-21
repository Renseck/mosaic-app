use std::future::Future;
use yew::prelude::*;
use crate::api::client::ApiError;

#[derive(Clone, PartialEq)]
pub struct FetchState<T: Clone + PartialEq> {
    pub data:    Option<T>,
    pub loading: bool,
    pub error:   Option<String>,
}

impl<T: Clone + PartialEq> FetchState<T> {
    pub fn loading() -> Self { Self { data: None, loading: true,  error: None } }
    pub fn idle()    -> Self { Self { data: None, loading: false, error: None } }
}

/* ============================================================================================== */
/// Simple data-fetch hook.
///
/// Returns `(state, reload)`. `reload` is a `Callback<()>` that re-runs the fetch.
///
/// ```ignore
/// let (state, reload) = use_api(|| api::dashboards::list_dashboards());
/// ```
#[hook]
pub fn use_api<T, F, Fut>(fetch_fn: F) -> (UseStateHandle<FetchState<T>>, Callback<()>)
where
    T: Clone + PartialEq + 'static,
    F: Fn() -> Fut + Clone + 'static,
    Fut: Future<Output = Result<T, ApiError>> + 'static,
{
    let state = use_state(FetchState::<T>::loading);

    // Initial fetch
    {
        let state    = state.clone();
        let fetch_fn = fetch_fn.clone();
        use_effect_with((), move |_| {
            let state = state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                state.set(match fetch_fn().await {
                    Ok(data) => FetchState { data: Some(data), loading: false, error: None },
                    Err(e)   => FetchState { data: None, loading: false, error: Some(e.to_string()) },
                });
            });
            || ()
        });
    }

    // Reload callback
    let reload = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            let state    = state.clone();
            let fetch_fn = fetch_fn.clone();
            // Preserve existing data while reloading (no flash)
            state.set(FetchState { data: state.data.clone(), loading: true, error: None });
            wasm_bindgen_futures::spawn_local(async move {
                state.set(match fetch_fn().await {
                    Ok(data) => FetchState { data: Some(data), loading: false, error: None },
                    Err(e)   => FetchState { data: None, loading: false, error: Some(e.to_string()) },
                });
            });
        })
    };

    (state, reload)
}
