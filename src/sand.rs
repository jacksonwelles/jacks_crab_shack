use std::cell::RefCell;
use std::convert::Infallible;
use std::f64::consts::PI;
use std::ops::Div;
use std::rc::Rc;

use utility::prelude::*;

use utility_macro::render_pipeline;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use leptos::logging::log;

use leptos_use::use_event_listener;

use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

render_pipeline!(AvalanchePipeline, "shaders/avalanche.frag");

render_pipeline!(DropPipeline, "shaders/drop_sand.frag");

render_pipeline!(ShadowPipeline, "shaders/shadow.frag");

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (mouse, set_mouse) = signal((0usize, 0i32, 0i32));
    let _ = use_event_listener(canvas_ref, leptos::ev::click, move |evt| {
        set_mouse.update(|tup| {
            tup.0 += 1;
            tup.1 = evt.offset_x();
            tup.2 = evt.offset_y();
        });
    });
    let (count, set_count) = signal(0);
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            canvas.set_width(512);
            canvas.set_height(512);
            let context = canvas
                .get_context("webgl2")
                .expect("get_context")
                .expect("object")
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            canvas_fill(context.clone(), count.into(), mouse.into());
        }
    });

    view! {
    <button
        on:click=move |_| *set_count.write() += 1
    >
        {move || {if count.get() % 2 == 0 {"STOP"} else {"START"}}}
    </button> <canvas node_ref=canvas_ref /> }
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    count: Signal<i32>,
    mouse: Signal<(usize, i32, i32)>,
) {
    let quad_vert_shader = compile_shader(
        &context,
        GL::VERTEX_SHADER,
        include_str!("shaders/quad.vert"),
    )
    .unwrap();

    let quad_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/quad.frag"),
    )
    .unwrap();

    let avalanche_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/avalanche.frag"),
    )
    .unwrap();

    let shadow_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/shadow.frag"),
    )
    .unwrap();

    let drop_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/drop_sand.frag"),
    )
    .unwrap();

    let window_w = context.drawing_buffer_width() as usize;
    let window_h = context.drawing_buffer_height() as usize;

    let sand_w = 4;
    let sand_h = 4;

    let window_texel_size = (1.0 / window_w as f32, 1.0 / window_h as f32);

    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let avalanche_program = Program::create(&context, &quad_vert_shader, &avalanche_frag_shader);
    let shadow_program = Program::create(&context, &quad_vert_shader, &shadow_frag_shader);
    let drop_program = Program::create(&context, &quad_vert_shader, &drop_frag_shader);

    let mut avalanche_pipeline = AvalanchePipeline::create(&context, avalanche_program);
    let mut shadow_pipeline = ShadowPipeline::create(&context, shadow_program);
    let mut drop_pipeline = DropPipeline::create(&context, drop_program);

    let mut sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));

    let (next_fame, set_next_frame) = signal(());

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    let mut prev_time = None::<f64>;
    let mut angle = 0.0;

    let quad = Rc::new(Quad::create(&context));
    {
        let context = context.clone();
        let sand = sand.clone();
        let quad = quad.clone();
        Effect::new(move || {
            let (_, mouse_x, mouse_y) = mouse.get();
            let pos: (f32, f32) = (
                mouse_x as f32 / window_w as f32,
                1.0 - mouse_y as f32 / window_h as f32,
            );
            drop_pipeline.set_arguments(
                &context,
                sand.borrow().read(),
                sand.borrow().read().texel_size(),
                255.0,
                0.6,
                pos,
            );
            quad.blit(Some(&sand.borrow().write()));
            sand.borrow_mut().swap();
        });
    }

    Effect::new(move || {
        next_fame.get();
        let now = window().performance().unwrap().now();

        if count.get() % 2 == 0 {
            angle = now % 20000.0 * (PI / 10000.0);
        }

        shadow_pipeline.set_arguments(
            &context,
            sand.borrow().read(),
            window_texel_size,
            (angle.cos() as f32, angle.sin() as f32),
            30f32.to_radians().tan(),
            255.0,
        );
        quad.blit(None);

        request_animation_frame(move || {
            *set_next_frame.write();
        });
    });
}

fn make_sand(context: &WebGl2RenderingContext, width: usize, height: usize) -> SwappableTexture {
    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::R8,
        width as i32,
        height as i32,
        0,
        GL::RED,
        GL::UNSIGNED_BYTE,
        None::<Infallible>,
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
}
