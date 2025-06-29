use super::common::*;

use std::cell::RefCell;
use std::ops::Div;
use std::rc::Rc;

use leptos::html::Canvas;
use leptos::prelude::*;

use leptos::wasm_bindgen::prelude::*;

use leptos_use::UseMouseReturn;
use web_sys::WebGl2RenderingContext;
use web_sys::console;

use leptos_use::{UseMouseOptions, use_mouse_with_options};

type GL = WebGl2RenderingContext;

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let mouse = use_mouse_with_options(UseMouseOptions::default().target(canvas_ref));
    let mouse_rc = Rc::new(mouse);
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
            canvas_fill(context, mouse_rc.clone());
        }
        console::log_1(&"Running Main Effect".into());
    });

    view! { <canvas node_ref=canvas_ref /> }
}

fn canvas_fill(context: WebGl2RenderingContext, mouse: Rc<UseMouseReturn>) {
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

    let force_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/force.frag"),
    )
    .unwrap();

    let advect_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/advect.frag"),
    )
    .unwrap();

    let boundary_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/boundary.frag"),
    )
    .unwrap();

    let divergence_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/divergence.frag"),
    )
    .unwrap();

    let gradient_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/gradient.frag"),
    )
    .unwrap();

    let jacobi_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/jacobi.frag"),
    )
    .unwrap();

    let force_program = Program::create(&context, &quad_vert_shader, &force_frag_shader);
    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let advect_program = Program::create(&context, &quad_vert_shader, &advect_frag_shader);
    let boundary_program = Program::create(&context, &quad_vert_shader, &boundary_frag_shader);
    let divergence_program = Program::create(&context, &quad_vert_shader, &divergence_frag_shader);
    let gradient_program = Program::create(&context, &quad_vert_shader, &gradient_frag_shader);
    let jacobi_program = Program::create(&context, &quad_vert_shader, &jacobi_frag_shader);

    let boundary = make_boundary_offsets(&context);
    let temp_texture = make_temp_texture(&context);
    let mut pressure = make_pressure_texture(&context);
    let mut velocity = make_initial_velocity(&context);
    let mut dye = make_initial_dye(&context);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let timescale = 1.0;
    let viscosity = 0.5;
    let diffusion_alpha = (
        1.0.div(velocity.texel_size().x.powi(2))
            .div(viscosity * timescale),
        1.0.div(velocity.texel_size().y.powi(2))
            .div(viscosity * timescale),
    );
    let diffusion_beta = (
        1.0.div(diffusion_alpha.0 + 4.0),
        1.0.div(diffusion_alpha.1 + 4.0),
    );

    let pressure_alpha = (
        -1.0.div(velocity.texel_size().x.powi(2)),
        -1.0.div(velocity.texel_size().y.powi(2)),
    );

    let pressure_beta = (0.25f32, 0.25f32);
    let mut prev_mouse = (0.0, 1.0);

    let quad = Quad::create(&context);

    *g.borrow_mut() = Some(Closure::new(move || {
        // Compute Boundary
        context.use_program(Some(boundary_program.program()));
        context.uniform1f(
            boundary_program.uniforms().get("u_scale").unwrap().into(),
            -1.0,
        );
        context.uniform1i(
            boundary_program.uniforms().get("u_target").unwrap().into(),
            velocity.read().attach(0),
        );
        context.uniform1i(
            boundary_program
                .uniforms()
                .get("u_boundary_offsets")
                .unwrap()
                .into(),
            boundary.attach(1),
        );
        context.uniform2f(
            boundary_program
                .uniforms()
                .get("u_texel_size")
                .unwrap()
                .into(),
            velocity.texel_size().x,
            velocity.texel_size().y,
        );
        quad.blit(Some(velocity.write()));
        velocity.swap();

        // Advect Velocity
        context.use_program(Some(advect_program.program()));
        context.uniform1f(
            advect_program.uniforms().get("u_timestep").unwrap().into(),
            timescale,
        );
        context.uniform2f(
            advect_program
                .uniforms()
                .get("u_texel_size")
                .unwrap()
                .into(),
            velocity.texel_size().x,
            velocity.texel_size().y,
        );
        context.uniform1i(
            advect_program.uniforms().get("u_target").unwrap().into(),
            velocity.read().attach(0),
        );
        context.uniform1i(
            advect_program.uniforms().get("u_velocity").unwrap().into(),
            velocity.read().attach(1),
        );
        quad.blit(Some(velocity.write()));
        velocity.swap();

        // Advect Dye
        context.use_program(Some(advect_program.program()));
        context.uniform1i(
            advect_program.uniforms().get("u_target").unwrap().into(),
            dye.read().attach(0),
        );
        context.uniform1i(
            advect_program.uniforms().get("u_velocity").unwrap().into(),
            velocity.read().attach(1),
        );
        context.uniform2f(
            advect_program
                .uniforms()
                .get("u_texel_size")
                .unwrap()
                .into(),
            velocity.texel_size().x,
            velocity.texel_size().y,
        );
        context.uniform1f(
            advect_program.uniforms().get("u_timestep").unwrap().into(),
            timescale,
        );
        quad.blit(Some(dye.write()));
        dye.swap();

        // Add impulse
        let cur_mouse = (
            mouse.x.get_untracked() / 512.0,
            1.0 - mouse.y.get_untracked() / 512.0,
        );

        console::log_1(&format!("x: {}, y: {}", mouse.x.get_untracked(), mouse.y.get_untracked()).into());
        context.use_program(Some(force_program.program()));
        context.uniform2f(
            force_program.uniforms().get("u_location").unwrap().into(),
            cur_mouse.0 as f32,
            cur_mouse.1 as f32,
        );
        context.uniform2f(
            force_program.uniforms().get("u_direction").unwrap().into(),
            (cur_mouse.0 - prev_mouse.0) as f32,
            (cur_mouse.1 - prev_mouse.1) as f32,
        );
        context.uniform1f(force_program.uniforms().get("u_radius"), 1.0 / 24.0);
        context.uniform1f(force_program.uniforms().get("u_scale"), 7.0);
        context.uniform1i(
            force_program.uniforms().get("u_velocity"),
            velocity.read().attach(0),
        );
        quad.blit( Some(velocity.write()));
        velocity.swap();
        prev_mouse = cur_mouse;

        // Diffuse
        // context.use_program(Some(jacobi_program.program()));
        // temp_texture.copy_from(velocity.read()).unwrap();
        // context.uniform2f(
        //     jacobi_program.uniforms().get("u_alpha").unwrap().into(),
        //     diffusion_alpha.0,
        //     diffusion_alpha.1,
        // );
        // context.uniform2f(
        //     jacobi_program.uniforms().get("u_r_beta").unwrap().into(),
        //     diffusion_beta.0,
        //     diffusion_beta.1,
        // );
        // context.uniform1i(
        //     jacobi_program.uniforms().get("u_initial").unwrap().into(),
        //     temp_texture.attach(0),
        // );
        // for _ in [0..30] {
        //     context.uniform1i(
        //         jacobi_program.uniforms().get("u_solution").unwrap().into(),
        //         velocity.read().attach(1),
        //     );
        //     quad.blit( Some(velocity.write()));
        //     velocity.swap();
        // }

        // Compute Divergance
        context.use_program(Some(divergence_program.program()));
        context.uniform2f(
            divergence_program
                .uniforms()
                .get("u_texel_size")
                .unwrap()
                .into(),
            velocity.texel_size().x,
            velocity.texel_size().y,
        );
        context.uniform1i(
            divergence_program
                .uniforms()
                .get("u_velocity")
                .unwrap()
                .into(),
            velocity.read().attach(0),
        );
        quad.blit(Some(pressure.write()));
        pressure.swap();

        // Compute Pressure
        context.use_program(Some(boundary_program.program()));
        context.uniform1f(
            boundary_program.uniforms().get("u_scale").unwrap().into(),
            1.0,
        );

        temp_texture.copy_from(pressure.read()).unwrap();

        context.use_program(Some(jacobi_program.program()));
        context.uniform2f(
            jacobi_program.uniforms().get("u_alpha").unwrap().into(),
            pressure_alpha.0,
            pressure_alpha.1,
        );
        context.uniform2f(
            jacobi_program.uniforms().get("u_r_beta").unwrap().into(),
            pressure_beta.0,
            pressure_beta.1,
        );

        for _ in 0..2 {
            context.use_program(Some(boundary_program.program()));
            context.uniform1i(
                boundary_program.uniforms().get("u_target").unwrap().into(),
                pressure.read().attach(0),
            );
            context.uniform1i(
                boundary_program
                    .uniforms()
                    .get("u_boundary_offsets")
                    .unwrap()
                    .into(),
                boundary.attach(1),
            );
            quad.blit(Some(pressure.write()));
            pressure.swap();

            context.use_program(Some(jacobi_program.program()));
            context.uniform1i(
                jacobi_program.uniforms().get("u_initial").unwrap().into(),
                pressure.read().attach(0),
            );
            context.uniform1i(
                jacobi_program.uniforms().get("u_solution").unwrap().into(),
                pressure.read().attach(1),
            );
            quad.blit(Some(pressure.write()));
            pressure.swap();
        }

        // // Reapply Boundaries
        // context.use_program(Some(boundary_program.program()));
        // context.uniform1f(
        //     boundary_program.uniforms().get("u_scale").unwrap().into(),
        //     -1.0,
        // );
        // context.uniform1i(
        //     boundary_program.uniforms().get("u_target").unwrap().into(),
        //     velocity.read().attach(0),
        // );
        // context.uniform1i(
        //     boundary_program
        //         .uniforms()
        //         .get("u_boundary_offsets")
        //         .unwrap()
        //         .into(),
        //     boundary.attach(1),
        // );
        // quad.blit(Some(velocity.write()));
        // velocity.swap();

        // // Gradient Subtraction
        // context.use_program(Some(gradient_program.program()));
        // context.uniform2f(
        //     gradient_program
        //         .uniforms()
        //         .get("u_texel_size")
        //         .unwrap()
        //         .into(),
        //     velocity.texel_size().x,
        //     velocity.texel_size().y,
        // );
        // context.uniform1i(
        //     gradient_program
        //         .uniforms()
        //         .get("u_velocity")
        //         .unwrap()
        //         .into(),
        //     velocity.read().attach(0),
        // );
        // context.uniform1i(
        //     gradient_program
        //         .uniforms()
        //         .get("u_pressure")
        //         .unwrap()
        //         .into(),
        //     pressure.read().attach(1),
        // );
        // quad.blit(Some(velocity.write()));
        // velocity.swap();

        // draw the dye
        context.use_program(Some(quad_program.program()));
        context.uniform1i(
            quad_program.uniforms().get("u_texture").unwrap().into(),
            pressure.read().attach(0),
        );

        quad.blit(None);

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

fn make_temp_texture(context: &WebGl2RenderingContext) -> BufferedTexture {
    BufferedTexture::create(
        &context,
        GL::TEXTURE_2D,
        0,
        GL::RG16F,
        128,
        128,
        0,
        GL::RG,
        GL::FLOAT,
        None::<ArrayView<f32>>,
        &[
            (GL::TEXTURE_MIN_FILTER, GL::LINEAR),
            (GL::TEXTURE_MAG_FILTER, GL::LINEAR),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    )
}

fn make_pressure_texture(context: &WebGl2RenderingContext) -> SwappableTexture {
    SwappableTexture::create(
        &context,
        GL::TEXTURE_2D,
        0,
        GL::RG16F,
        128,
        128,
        0,
        GL::RG,
        GL::FLOAT,
        None::<ArrayView<f32>>,
        &[
            (GL::TEXTURE_MIN_FILTER, GL::LINEAR),
            (GL::TEXTURE_MAG_FILTER, GL::LINEAR),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    )
}

fn make_boundary_offsets(context: &WebGl2RenderingContext) -> BufferedTexture {
    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;
    const VALUES_PER_PIXEL: usize = 2;
    const TEX_DATA_SIZE: usize = WIDTH * HEIGHT * VALUES_PER_PIXEL;
    let mut texture_data: [f32; TEX_DATA_SIZE] = [0.0; TEX_DATA_SIZE];

    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        let pixel = i / VALUES_PER_PIXEL;
        let row = pixel / WIDTH;
        let col = pixel % WIDTH;
        *elem = match (pos, row, col) {
            (0, .., 0) => 1.0,
            (0, .., x) if x == WIDTH - 1 => -1.0,
            (1, 0, ..) => 1.0,
            (1, y, ..) if y == HEIGHT - 1 => -1.0,
            _ => 0.0,
        }
    }

    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RG16F,
        WIDTH as i32,
        HEIGHT as i32,
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

fn make_initial_dye(context: &WebGl2RenderingContext) -> SwappableTexture {
    const WIDTH: usize = 512;
    const HEIGHT: usize = 512;
    const VALUES_PER_PIXEL: usize = 3;
    const TEX_DATA_SIZE: usize = WIDTH * HEIGHT * VALUES_PER_PIXEL;
    let mut texture_data: [u8; TEX_DATA_SIZE] = [0; TEX_DATA_SIZE];
    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        let pixel = i / VALUES_PER_PIXEL;
        let row = pixel / WIDTH;
        let col = pixel % WIDTH;
        let dist = ((row as f32 - 256.0).powi(2) + (col as f32 - 256.0).powi(2)).sqrt();
        if dist < 128.0 {
            *elem = (match pos {
                1 => (128.0 - dist) / 128.0,
                2 => 1.0,
                _ => 0.0,
            } * 255.0) as u8;
        }
    }

    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGB,
        WIDTH as i32,
        HEIGHT as i32,
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

fn make_initial_velocity(context: &WebGl2RenderingContext) -> SwappableTexture {
    const WIDTH: usize = 128;
    const HEIGHT: usize = 128;
    const VALUES_PER_PIXEL: usize = 2;
    const TEX_DATA_SIZE: usize = WIDTH * HEIGHT * VALUES_PER_PIXEL;
    let texture_data: [f32; TEX_DATA_SIZE] = [0.0; TEX_DATA_SIZE];

    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RG16F,
        WIDTH as i32,
        HEIGHT as i32,
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
