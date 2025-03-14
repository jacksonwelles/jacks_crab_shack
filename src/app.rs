use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = signal(0);
    let double_count = move || count.get() * 2;

    return view! {
        <button
            on:click=move |_| {*set_count.write() += 1; }
        >
            "Click to Move" {count}
        </button>
        <ProgressBar progress= move || count.get()/>
        <ProgressBar progress=double_count/>
    }
}

#[component]
fn ProgressBar(
    #[prop(default = 100)]
    max: u16,
    progress: impl Fn() -> i32 + Send + Sync + 'static
) -> impl IntoView {
    view! {
        <progress
            max=max
            value=progress
        />
    }
}
