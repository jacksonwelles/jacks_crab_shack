mod fluids;
mod sand;

use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router_macro::path;

#[component]
fn HomeView() -> impl IntoView {


    view! {
        <h1 style:margin="40px">"Jackson Welles"</h1>
        <A href="/jacks_crab_shack/fluids" style:margin="40px" style:font-size="40px">
            "Fluids"
        </A>
        <br />
        <A href="/jacks_crab_shack/sand" style:margin="40px" style:font-size="40px">
            "Sand"
        </A>
        <br />
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Router base="/jacks_crab_shack">
            // ordinary <a> elements can be used for client-side navigation
            // using <A> has two effects:
            // 1) ensuring that relative routing works properly for nested routes
            // 2) setting the `aria-current` attribute on the current link,
            // for a11y and styling purposes

            <Routes transition=true fallback=|| "This page could not be found.">
                <Route path=path!("/sand") view=sand::App />
                <Route path=path!("/fluids") view=fluids::App />
                <Route path=path!("/") view=HomeView />
            </Routes>
        </Router>
    }
}
