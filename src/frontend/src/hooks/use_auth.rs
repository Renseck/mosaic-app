use yew::prelude::*;
use crate::context::auth_context::AuthContext;

/// Convenience hook — panics if used outside `<AuthProvider>`.
#[hook]
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>().expect("use_auth must be called inside AuthProvider")
}