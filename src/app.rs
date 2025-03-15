use leptos::wasm_bindgen::prelude::*;
use leptos::{html::Canvas, prelude::*};
use std::f64;

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (count, set_count) = signal(0);

    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            let context = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            context.reset();
            context.begin_path();

            // Draw the outer circle.
            context
                .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
                .unwrap();
            
            // Draw the mouth.
            if count.get() % 2 == 0 {    
                context.move_to(100.0, 80.0);
                context.arc(75.0, 80.0, 25.0, 0.0,  f64::consts::PI).unwrap();
            } else {
                context.move_to(50.0, 105.0);
                context.arc(75.0, 105.0, 25.0, f64::consts::PI,  0.0).unwrap();
            }


            // Draw the left eye.
            context.move_to(65.0, 65.0);
            context
                .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
                .unwrap();

            // Draw the right eye.
            context.move_to(95.0, 65.0);
            context
                .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
                .unwrap();

            context.stroke();
        }
    });

    view! {
      <button on:click=move |_| {*set_count.write() += 1;}> "Flip" </button>
      <canvas node_ref=canvas_ref/>
    }
}
