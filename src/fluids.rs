use super::common::*;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::ops::Add;
use std::ops::Mul;
use std::rc::Rc;

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
    context.get_extension("EXT_color_buffer_float").unwrap();
    context.get_extension("OES_texture_float_linear").unwrap();
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

    let advect_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/advect.frag"),
    )
    .unwrap();

    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let advect_program = Program::create(&context, &quad_vert_shader, &advect_frag_shader);

    let velocity = make_initial_velocity(&context);
    let mut dye = make_initial_dye(&context);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        context.use_program(Some(quad_program.program()));
        context.uniform1i(
            quad_program.uniforms().get("u_texture"),
            dye.read().attach(0),
        );

        blit(&context, None);

        context.use_program(Some(advect_program.program()));
        context.uniform1i(
            advect_program.uniforms().get("u_target"),
            dye.read().attach(0),
        );
        context.uniform1i(
            advect_program.uniforms().get("u_velocity"),
            velocity.attach(1),
        );
        context.uniform2f(
            advect_program.uniforms().get("u_velocity_texel_size"),
            velocity.texel_size().x,
            velocity.texel_size().y,
        );
        context.uniform1f(advect_program.uniforms().get("u_timestep"), 1.0);

        blit(&context, Some(dye.write()));

        dye.swap();

        window()
            .request_animation_frame(
                (f.borrow().as_ref().unwrap() as &Closure<dyn FnMut()>)
                    .as_ref()
                    .unchecked_ref(),
            )
            .expect("requestAnimationFrame failed");
    }));

    window()
        .request_animation_frame(
            (g.borrow().as_ref().unwrap() as &Closure<dyn FnMut()>)
                .as_ref()
                .unchecked_ref(),
        )
        .expect("requestAnimationFrame failed");
}

fn make_initial_dye(context: &WebGl2RenderingContext) -> SwappableTexture {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 256;
    const VALUES_PER_PIXEL: usize = 3;
    const TEX_DATA_SIZE: usize = WIDTH * HEIGHT * VALUES_PER_PIXEL;
    const TEX_FREQUENCY: f64 = (10.0 * PI) / WIDTH as f64;
    let mut texture_data: [u8; TEX_DATA_SIZE] = [0; TEX_DATA_SIZE];
    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        let pixel = i / VALUES_PER_PIXEL;
        if pos == 2 {
            *elem = (pixel as f64).mul(TEX_FREQUENCY).sin().add(1.0).mul(128.0) as u8;
        }
    }

    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGB,
        256,
        256,
        0,
        GL::RGB,
        GL::UNSIGNED_BYTE,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::LINEAR),
            (GL::TEXTURE_MAG_FILTER, GL::LINEAR),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    );
}

fn make_initial_velocity(context: &WebGl2RenderingContext) -> BufferedTexture {
    const WIDTH: usize = 32;
    const HEIGHT: usize = 32;
    const VALUES_PER_PIXEL: usize = 2;
    const TEX_DATA_SIZE: usize = WIDTH * HEIGHT * VALUES_PER_PIXEL;
    let mut texture_data: [f32; TEX_DATA_SIZE] = [0.0; TEX_DATA_SIZE];

    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        if pos == 0 {
            *elem = 1.0
        }
    }

    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RG16F,
        32,
        32,
        0,
        GL::RG,
        GL::FLOAT,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::LINEAR),
            (GL::TEXTURE_MAG_FILTER, GL::LINEAR),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    );
}
