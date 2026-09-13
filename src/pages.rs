mod sand;
mod fluids;

use leptos::prelude::*;
use leptos_router::{
    components::{
        Route, Router,
        Routes, A,
    },
};
use leptos_router_macro::path;

#[component]
fn HomeView() -> impl IntoView {
    view !{
        <A href="/fluids">"Fluids"</A>
        <br />
        <A href="/sand">"Sand"</A>
        <br />
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Router>
            <main>
                // ordinary <a> elements can be used for client-side navigation
                // using <A> has two effects:
                // 1) ensuring that relative routing works properly for nested routes
                // 2) setting the `aria-current` attribute on the current link,
                // for a11y and styling purposes


                <Routes transition=true fallback=|| "This page could not be found.">
                    <Route path=path!("sand") view=sand::App />
                    <Route path=path!("fluids") view=fluids::App />
                    <Route path=path!("/") view=HomeView />
                </Routes>
            </main>
        </Router>
    }
}