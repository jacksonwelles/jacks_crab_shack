use super::common::*;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
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
            canvas_fill(context);
        }
    });

    view! { <canvas node_ref=canvas_ref /> }
}

fn canvas_fill(context: WebGl2RenderingContext) {
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

    let life_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/life.frag"),
    )
    .unwrap();
    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let life_program = Program::create(&context, &quad_vert_shader, &life_frag_shader);

    let mut game_board = make_game_board(&context);

    let (frame_count, set_frame_count) = signal(());

    request_animation_frame(move || {
        *set_frame_count.write();
    });

    let mut prev_time = None::<f64>;

    let quad = Quad::create(&context);

    Effect::new(move || {
        frame_count.read();
        let now = window().performance().unwrap().now();
        if !prev_time.is_some() || now - prev_time.unwrap() > 50.0 {
            prev_time = Some(now);

            context.use_program(Some(quad_program.program()));
            context.uniform1i(
                quad_program.uniforms().get("u_texture"),
                game_board.read().attach(0),
            );
            quad.blit(None);

            context.use_program(Some(life_program.program()));
            context.uniform1i(
                life_program.uniforms().get("u_texture"),
                game_board.read().attach(0),
            );
            context.uniform2f(
                life_program.uniforms().get("u_texel_size"),
                game_board.read().texel_size().0,
                game_board.read().texel_size().1,
            );
            quad.blit(Some(&game_board.write()));

            game_board.swap();
        }
        request_animation_frame(move || {
            *set_frame_count.write();
        });
    });
}

fn make_game_board(context: &WebGl2RenderingContext) -> SwappableTexture {
    let mut texture_data: [u8; 4096] = [0; 4096];

    for (i, elem) in texture_data.iter_mut().enumerate() {
        *elem = (i % 4 == 3) as u8 * 255;
    }

    for elem in texture_data[1332..1344].iter_mut() {
        *elem = 255;
    }

    for elem in texture_data[1212..1216].iter_mut() {
        *elem = 255;
    }

    for elem in texture_data[1080..1084].iter_mut() {
        *elem = 255;
    }

    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGBA,
        32,
        32,
        0,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
}
