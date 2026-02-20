use std::rc::Rc;
use yew::prelude::*;
use crate::models::User;
use crate::api::auth;

/* ============================================================================================== */
/*                                              State                                             */
/* ============================================================================================== */

#[derive(Debug, Clone, PartialEq)]
pub struct AuthState {
    pub user: Option<User>,
    /// True while the initial /api/auth/me check is in flight.
    pub loading: bool,
}

/* ============================================================================================== */
/*                                             Actions                                            */
/* ============================================================================================== */

pub enum AuthAction {
    SetUser(User),
    ClearUser,
    SetLoading(bool),
}

/* ============================================================================================== */
/*                                             Reducer                                            */
/* ============================================================================================== */

impl Reducible for AuthState {
    type Action = AuthAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AuthAction::SetUser(user) => Rc::new(AuthState {
                user: Some(user),
                loading: false,
            }),
            AuthAction::ClearUser => Rc::new(AuthState {
                user: None,
                loading: false,
            }),
            AuthAction::SetLoading(loading) => Rc::new(AuthState {
                user: self.user.clone(),
                loading,
            }),
        }
    }
}


/* ============================================================================================== */
/*                                          Context type                                          */
/* ============================================================================================== */

pub type AuthContext = UseReducerHandle<AuthState>;

/* ============================================================================================== */
/*                                       Provider component                                       */
/* ============================================================================================== */

#[derive(Properties, PartialEq)]
pub struct AuthProviderProps {
    pub children: Children,
}

#[function_component(AuthProvider)]
pub fn auth_provider(props: &AuthProviderProps) -> Html {
    let state = use_reducer(|| AuthState {
        user: None,
        loading: true,
    });

    // On mount: check whether the browser already has a valid session.
    {
        let state = state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match auth::me().await {
                    Ok(user) => state.dispatch(AuthAction::SetUser(user)),
                    Err(_) => state.dispatch(AuthAction::ClearUser),
                }
            });
            || ()
        });
    }

    html! {
        <ContextProvider<AuthContext> context={state}>
            { for props.children.iter() }
        </ContextProvider<AuthContext>>
    }
}