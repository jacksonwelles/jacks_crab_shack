use std::cell::RefCell;
use std::convert::Infallible;
use std::f64::consts::PI;
use std::rc::Rc;

use utility::prelude::*;

use utility_macro::render_pipeline;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use leptos_use::use_event_listener;

use web_sys::WebGl2RenderingContext;
use web_sys::console;

type GL = WebGl2RenderingContext;

render_pipeline!(AvalanchePipeline, "shaders/avalanche.frag");

render_pipeline!(DropPipeline, "shaders/drop_sand.frag");

render_pipeline!(ShadowPipeline, "shaders/optimized_shadow.frag");

render_pipeline!(LookaheadPipeline, "shaders/precompute_shadow.frag");

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (mouse, set_mouse) = signal((false, 0i32, 0i32));
    let _ = use_event_listener(canvas_ref, leptos::ev::mousedown, move |evt| {
        *set_mouse.write() = (true, evt.offset_x(), evt.offset_y());
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::mouseup, move |evt| {
        *set_mouse.write() = (false, evt.offset_x(), evt.offset_y());
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::mousemove, move |evt| {
        set_mouse.update(|tup| {
            tup.1 = evt.offset_x();
            tup.2 = evt.offset_y();
        });
    });
    let (sun_move, set_sun_move) = signal(0);
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
            canvas_fill(context.clone(), sun_move.into(), mouse.into());
        }
    });

    view! {
     <canvas node_ref=canvas_ref />
     <br/>
     <button
        on:click=move |_| *set_sun_move.write() += 1
    >
        {move || {if sun_move.get() % 2 == 0 {"STOP"} else {"START"}}}
    </button> }
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    sun_move: Signal<i32>,
    mouse: Signal<(bool, i32, i32)>,
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
        include_str!("shaders/optimized_shadow.frag"),
    )
    .unwrap();

    let drop_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/drop_sand.frag"),
    )
    .unwrap();

    let lookahead_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/precompute_shadow.frag"),
    )
    .unwrap();

    let window_w = context.drawing_buffer_width() as usize;
    let window_h = context.drawing_buffer_height() as usize;

    let sand_w = window_w;
    let sand_h = window_h;
    let scale = 4.0f32;

    let window_texel_size = (1.0 / window_w as f32, 1.0 / window_h as f32);

    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let avalanche_program = Program::create(&context, &quad_vert_shader, &avalanche_frag_shader);
    let shadow_program = Program::create(&context, &quad_vert_shader, &shadow_frag_shader);
    let drop_program = Program::create(&context, &quad_vert_shader, &drop_frag_shader);
    let lookahead_program = Program::create(&context, &quad_vert_shader, &lookahead_frag_shader);

    let mut avalanche_pipeline = AvalanchePipeline::create(&context, avalanche_program);
    let mut shadow_pipeline = ShadowPipeline::create(&context, shadow_program);
    let mut drop_pipeline = DropPipeline::create(&context, drop_program);
    let mut lookahead_pipeline = LookaheadPipeline::create(&context, lookahead_program);

    let sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));
    let lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, window_w, window_h,
    )));

    let (next_frame, set_next_frame) = signal(());

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    let (signal_lookahead, set_signal_lookahead) = signal(());
    let (signal_drop, set_signal_drop) = signal(());
    let (signal_avalanche, set_signal_avalanche) = signal(());
    let (angle, set_angle) = signal(0.0);

    let mut prev_avalance = None::<f64>;
    let mut prev_time = None::<f64>;

    Effect::new(move || {
        next_frame.get();
        let now = window().performance().unwrap().now();
        if sun_move.get_untracked() % 2 == 0 {
            if prev_time.is_some() {
                *set_angle.write() += (now - prev_time.unwrap()) % 20000.0 * (PI / 10000.0);
            }
            set_signal_lookahead.write();
        }
        if mouse.get_untracked().0 {
            set_signal_drop.write();
        }
        set_signal_avalanche.write();
        prev_time = Some(now);
        request_animation_frame(move || {
            *set_next_frame.write();
        });
    });

    let quad = Rc::new(Quad::create(&context));

    // Drop sand
    let mut prev_drop = None::<f64>;
    {
        let context = context.clone();
        let sand = sand.clone();
        let quad = quad.clone();
        Effect::new(move || {
            signal_drop.get();
            let now = window().performance().unwrap().now();
            if prev_drop.is_some() && now - prev_drop.unwrap() < 16.0 {
                return;
            }
            prev_drop = Some(now);
            let (_, mouse_x, mouse_y) = mouse.get_untracked();
            let pos: (f32, f32) = (
                mouse_x as f32 / window_w as f32,
                1.0 - mouse_y as f32 / window_h as f32,
            );
            drop_pipeline.set_arguments(
                &context,
                sand.borrow().read(),
                sand.borrow().read().texel_size(),
                255.0,
                60.0,
                pos,
            );
            quad.blit(Some(&sand.borrow().write()));
            sand.borrow_mut().swap();
            set_signal_lookahead.write();
            console::log_1(&"Dropping sand".into());
        });
    }

    // Update the lookahead texture
    let mut prev_lookahead = None::<f64>;
    {
        let context = context.clone();
        let sand = sand.clone();
        let quad = quad.clone();
        let lookahead = lookahead.clone();
        Effect::new(move || {
            signal_lookahead.get();
            let now = window().performance().unwrap().now();
            if prev_lookahead.is_some() && now - prev_lookahead.unwrap() < 16.0 {
                return;
            }
            prev_lookahead = Some(now);
            let direction = (
                angle.get_untracked().cos() as f32,
                angle.get_untracked().sin() as f32,
            );
            lookahead_pipeline.set_arguments(
                &context,
                sand.borrow().read(),
                scale,
                direction,
                lookahead.borrow().read().texel_size(),
                0.0,
            );

            quad.blit(Some(lookahead.borrow().write()));
            lookahead.borrow_mut().swap();
            for i in 1..4 {
                lookahead_pipeline.set_arguments(
                    &context,
                    lookahead.borrow().read(),
                    scale,
                    direction,
                    lookahead.borrow().read().texel_size(),
                    i as f32,
                );
                quad.blit(Some(lookahead.borrow().write()));
                lookahead.borrow_mut().swap();
            }
        });
    }

    Effect::new(move || {
        next_frame.get();
        let direction = (
            angle.get_untracked().cos() as f32,
            angle.get_untracked().sin() as f32,
        );

        shadow_pipeline.set_arguments(
            &context,
            sand.borrow().read(),
            lookahead.borrow().read(),
            scale,
            (window_texel_size.0, window_texel_size.1, 1.0 / 255.0),
            (direction.0, direction.1, 30f32.to_radians().tan()),
        );
        quad.blit(None);
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

fn make_shadow_lookahead(
    context: &WebGl2RenderingContext,
    width: usize,
    height: usize,
) -> SwappableTexture {
    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGBA8,
        width as i32,
        height as i32,
        0,
        GL::RGBA,
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
