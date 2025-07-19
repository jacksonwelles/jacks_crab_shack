use utility::prelude::*;

use std::cell::RefCell;
use std::cmp;
use std::ops::Div;
use std::ops::Sub;
use std::rc::Rc;

use leptos::html::Canvas;
use leptos::prelude::*;

use leptos::wasm_bindgen::prelude::*;

use leptos::wasm_bindgen::JsCast;
use leptos_use::UseMouseEventExtractor;
use leptos_use::UseMouseReturn;
use web_sys::HtmlElement;
use web_sys::MouseEvent;
use web_sys::Touch;
use web_sys::WebGl2RenderingContext;
use web_sys::console;

use leptos_use::{UseMouseCoordType, UseMouseOptions, use_mouse_with_options};

type GL = WebGl2RenderingContext;
#[derive(Clone)]
struct OffsetExtractor;
impl UseMouseEventExtractor for OffsetExtractor {
    fn extract_mouse_coords(&self, event: &MouseEvent) -> Option<(f64, f64)> {
        match event.buttons() % 2 {
            1 => Some((event.offset_x() as f64, event.offset_y() as f64)),
            _ => None,
        }
    }

    fn extract_touch_coords(&self, touch: &Touch) -> Option<(f64, f64)> {
        let element = touch
            .target()
            .unwrap()
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .clone();
        let rect = element.get_bounding_client_rect();
        Some((
            touch.client_x() as f64 - rect.x(),
            touch.client_y() as f64 - rect.y(),
        ))
    }
}

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let mouse = use_mouse_with_options(
        UseMouseOptions::default()
            .target(canvas_ref)
            .coord_type(UseMouseCoordType::Custom(OffsetExtractor)),
    );
    let mouse_rc = Rc::new(mouse);
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            canvas.set_width(1024);
            canvas.set_height(1024);
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

    view! {
        <h1 style:margin="40px">"WebGl Fluid Sim"</h1>
        <canvas style:padding="0px" style:touch-action="pinch-zoom" node_ref=canvas_ref />
        <h2 style:margin="40px">"Written by Jackson Welles"</h2>
        <h2 style:margin="40px">"Theory and shaders from GPU Gems: Chapter 38."</h2>
        <h2 style:margin="40px">
            "Abstractions and WebGL settings from Pavel Dobryakov (PavelDoGreat)"
        </h2>
    }
}

render_pipeline!(AdvectPipeline, "shaders/advect.frag");

render_pipeline!(ImpulsePipeline, "shaders/force.frag");

render_pipeline!(DivergencePipeline, "shaders/divergence.frag");

render_pipeline!(JacobiPipeline, "shaders/jacobi.frag");

render_pipeline!(BoundaryPipeline, "shaders/boundary.frag");

render_pipeline!(GradientSubtractPipeline, "shaders/gradient.frag");

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

    let window_w = context.drawing_buffer_width() as usize;
    let window_h = context.drawing_buffer_height() as usize;
    let sim_w = 128;
    let sim_h = 128;
    let dye_w = window_w;
    let dye_h = window_h;

    let force_radius = 1.0 / 24.0;
    let force_scale = 7.0;
    let timestep = 1.0;
    let viscosity = 0.5;
    let sim_texel_size = (1.0 / sim_w as f32, 1.0 / sim_h as f32);
    let diffusion_alpha = (
        1.0.div(sim_texel_size.0.powi(2)).div(viscosity * timestep),
        1.0.div(sim_texel_size.1.powi(2)).div(viscosity * timestep),
    );
    let diffusion_beta = (
        1.0.div(diffusion_alpha.0 + 4.0),
        1.0.div(diffusion_alpha.1 + 4.0),
    );

    let pressure_alpha = (
        -1.0.div(sim_texel_size.0.powi(2)),
        -1.0.div(sim_texel_size.1.powi(2)),
    );

    let pressure_beta = (0.25f32, 0.25f32);

    let force_program = Program::create(&context, &quad_vert_shader, &force_frag_shader);
    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let advect_program = Program::create(&context, &quad_vert_shader, &advect_frag_shader);
    let boundary_program = Program::create(&context, &quad_vert_shader, &boundary_frag_shader);
    let divergence_program = Program::create(&context, &quad_vert_shader, &divergence_frag_shader);
    let gradient_program = Program::create(&context, &quad_vert_shader, &gradient_frag_shader);
    let jacobi_program = Program::create(&context, &quad_vert_shader, &jacobi_frag_shader);

    let boundary_texture = make_boundary_offsets(sim_w, sim_h, &context);
    let temp_texture = make_blank::<BufferedTexture>(sim_w, sim_h, &context);
    let blank_texture = make_blank::<BufferedTexture>(sim_w, sim_h, &context);
    let mut pressure_texture = make_blank::<SwappableTexture>(sim_w, sim_h, &context);
    let mut velocity_texture = make_blank::<SwappableTexture>(sim_w, sim_h, &context);
    let mut dye_texture = make_initial_dye(dye_w, dye_h, &context);

    let mut advect_pipeline = AdvectPipeline::create(&context, advect_program);
    let mut impulse_pipeline = ImpulsePipeline::create(&context, force_program);
    let mut divergence_pipeline = DivergencePipeline::create(&context, divergence_program);
    let mut jacobi_pipeline = JacobiPipeline::create(&context, jacobi_program);
    let mut boundary_pipeline = BoundaryPipeline::create(&context, boundary_program);
    let mut gradient_pipeline = GradientSubtractPipeline::create(&context, gradient_program);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut prev_mouse: (f32, f32) = (0.0, 1.0);
    let mut prev_input_time = None::<f64>;
    let mut prev_frame = None::<f64>;

    let quad = Quad::create(&context);

    *g.borrow_mut() = Some(Closure::new(move || {
        let now = window().performance().unwrap().now();

        // Velocity Boundary
        boundary_pipeline.set_arguments(
            &context,
            velocity_texture.read(),
            &boundary_texture,
            velocity_texture.read().texel_size(),
            -1.0,
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Advect Velocity
        advect_pipeline.set_arguments(
            &context,
            velocity_texture.read(),
            velocity_texture.read(),
            velocity_texture.read().texel_size(),
            velocity_texture.read().texel_size(),
            timestep,
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Advect Dye
        advect_pipeline.set_arguments(
            &context,
            dye_texture.read(),
            velocity_texture.read(),
            dye_texture.read().texel_size(),
            velocity_texture.read().texel_size(),
            timestep,
        );
        quad.blit(Some(dye_texture.write()));
        dye_texture.swap();

        // Add impulse
        let cur_mouse: (f32, f32) = (
            mouse.x.get_untracked().div(window_w as f64) as f32,
            1.0.sub(mouse.y.get_untracked().div(window_h as f64)) as f32,
        );
        if cur_mouse != prev_mouse {
            if prev_input_time.is_some() && prev_frame.is_some() && prev_input_time >= prev_frame {
                impulse_pipeline.set_arguments(
                    &context,
                    velocity_texture.read(),
                    cur_mouse,
                    (cur_mouse.0 - prev_mouse.0, cur_mouse.1 - prev_mouse.1),
                    force_scale,
                    force_radius,
                );
                quad.blit(Some(velocity_texture.write()));
                velocity_texture.swap();
            }
            prev_input_time = Some(now);
        }
        prev_mouse = cur_mouse;

        // Diffuse
        temp_texture.copy_from(velocity_texture.read()).unwrap();
        velocity_texture
            .read()
            .copy_from(&blank_texture)
            .expect("failed to clear velocity");

        for _ in 0..30 {
            jacobi_pipeline.set_arguments(
                &context,
                velocity_texture.read(),
                &temp_texture,
                velocity_texture.read().texel_size(),
                diffusion_alpha,
                diffusion_beta,
            );
            quad.blit(Some(velocity_texture.write()));
            velocity_texture.swap();
        }

        // Compute Divergance
        divergence_pipeline.set_arguments(
            &context,
            velocity_texture.read(),
            velocity_texture.read().texel_size(),
        );
        quad.blit(Some(&temp_texture));

        // Compute Pressure
        pressure_texture
            .read()
            .copy_from(&blank_texture)
            .expect("failed to clear pressure");

        for _ in 0..40 {
            boundary_pipeline.set_arguments(
                &context,
                pressure_texture.read(),
                &boundary_texture,
                pressure_texture.read().texel_size(),
                1.0,
            );
            quad.blit(Some(pressure_texture.write()));
            pressure_texture.swap();

            jacobi_pipeline.set_arguments(
                &context,
                pressure_texture.read(),
                &temp_texture,
                pressure_texture.read().texel_size(),
                pressure_alpha,
                pressure_beta,
            );
            quad.blit(Some(pressure_texture.write()));
            pressure_texture.swap();
        }

        // Reapply Boundaries
        boundary_pipeline.set_arguments(
            &context,
            velocity_texture.read(),
            &boundary_texture,
            velocity_texture.read().texel_size(),
            -1.0,
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Gradient Subtraction
        gradient_pipeline.set_arguments(
            &context,
            velocity_texture.read(),
            pressure_texture.read(),
            velocity_texture.read().texel_size(),
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // draw the dye
        context.use_program(Some(quad_program.program()));
        context.uniform1i(
            quad_program.uniforms().get("u_texture").unwrap().into(),
            dye_texture.read().attach(0),
        );

        quad.blit(None);

        prev_frame = Some(now);

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

fn make_blank<T: FromJsView>(width: usize, height: usize, context: &WebGl2RenderingContext) -> T {
    const VALUES_PER_PIXEL: usize = 2;
    let tex_data_size = width as usize * height as usize * VALUES_PER_PIXEL;
    let texture_data = vec![0.0; tex_data_size];

    T::create(
        &context,
        GL::TEXTURE_2D,
        0,
        GL::RG32F,
        width as i32,
        height as i32,
        0,
        GL::RG,
        GL::FLOAT,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    )
}

fn make_boundary_offsets(
    width: usize,
    height: usize,
    context: &WebGl2RenderingContext,
) -> BufferedTexture {
    const VALUES_PER_PIXEL: usize = 2;
    let tex_data_size = width as usize * height as usize * VALUES_PER_PIXEL;
    let mut texture_data = vec![0.0; tex_data_size];

    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        let pixel = i / VALUES_PER_PIXEL;
        let row = pixel / width;
        let col = pixel % width;
        *elem = match (pos, row, col) {
            (0, .., 0) => 1.0,
            (0, .., x) if x == width - 1 => -1.0,
            (1, 0, ..) => 1.0,
            (1, y, ..) if y == height - 1 => -1.0,
            _ => 0.0,
        }
    }

    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RG32F,
        width as i32,
        height as i32,
        0,
        GL::RG,
        GL::FLOAT,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    );
}

fn make_initial_dye(
    width: usize,
    height: usize,
    context: &WebGl2RenderingContext,
) -> SwappableTexture {
    const VALUES_PER_PIXEL: usize = 4;
    let tex_data_size = width as usize * height as usize * VALUES_PER_PIXEL;
    let mut texture_data = vec![0.0; tex_data_size];
    for (i, elem) in texture_data.iter_mut().enumerate() {
        let pos = i % VALUES_PER_PIXEL;
        let pixel = i / VALUES_PER_PIXEL;
        let row = pixel / width;
        let col = pixel % width;
        let h_width = width as f32 / 2.0;
        let h_height = width as f32 / 2.0;
        let radius = cmp::min(width, height) as f32 / 4.0;
        let dist = ((row as f32 - h_width).powi(2) + (col as f32 - h_height).powi(2)).sqrt();
        *elem = match pos {
            1 if dist < radius => (radius - dist) / radius,
            2 if dist < radius => 1.0,
            3 => 1.0,
            _ => 0.0,
        };
    }

    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGBA32F,
        width as i32,
        height as i32,
        0,
        GL::RGBA,
        GL::FLOAT,
        Some(ArrayView::create(&texture_data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE),
            (GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE),
        ],
    );
}
