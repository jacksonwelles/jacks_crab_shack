use super::common::*;

use std::cell::RefCell;
use std::cmp;
use std::ops::Div;
use std::ops::Sub;
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

    view! { <canvas node_ref=canvas_ref /> }
}

struct BoundaryPipeline {
    program: Program,
    scale: f32,
    texel_size: (f32, f32),
}

impl BoundaryPipeline {
    const TARGET: i32 = 0;
    const OFFSET: i32 = 1;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> BoundaryPipeline {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program
                .uniforms()
                .get("u_target")
                .expect("missing argument")
                .into(),
            Self::TARGET,
        );
        context.uniform1i(
            program
                .uniforms()
                .get("u_boundary_offsets")
                .expect("missing argument")
                .into(),
            Self::OFFSET,
        );
        BoundaryPipeline {
            program,
            scale: 0.0,
            texel_size: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        scale: f32,
        boundary_texture: &BufferedTexture,
        target_texture: &BufferedTexture,
    ) -> () {
        debug_assert!(boundary_texture.texel_size() == target_texture.texel_size());
        context.use_program(Some(self.program.program()));
        target_texture.attach(Self::TARGET);
        boundary_texture.attach(Self::OFFSET);

        if self.scale != scale {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_scale")
                    .expect("missing argument")
                    .into(),
                scale,
            );
            self.scale = scale;
        }
        if self.texel_size != target_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_texel_size")
                    .expect("missing argument")
                    .into(),
                target_texture.texel_size().0,
                target_texture.texel_size().1,
            );
            self.texel_size = target_texture.texel_size();
        }
    }
}

struct AdvectPipeline {
    program: Program,
    timestep: f32,
    target_texel_size: (f32, f32),
    velocity_texel_size: (f32, f32),
}

impl AdvectPipeline {
    const TARGET: i32 = 0;
    const VELOCITY: i32 = 1;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> AdvectPipeline {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program
                .uniforms()
                .get("u_target")
                .expect("missing argument")
                .into(),
            Self::TARGET,
        );
        context.uniform1i(
            program
                .uniforms()
                .get("u_velocity")
                .expect("missing argument")
                .into(),
            Self::VELOCITY,
        );
        AdvectPipeline {
            program,
            timestep: 0.0,
            target_texel_size: (0.0, 0.0),
            velocity_texel_size: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        timestep: f32,
        velocity_texture: &BufferedTexture,
        target_texture: &BufferedTexture,
    ) -> () {
        context.use_program(Some(self.program.program()));
        velocity_texture.attach(Self::VELOCITY);
        target_texture.attach(Self::TARGET);

        if self.timestep != timestep {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_timestep")
                    .expect("missing argument")
                    .into(),
                timestep,
            );
            self.timestep = timestep;
        }

        if self.velocity_texel_size != velocity_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_velocity_texel_size")
                    .expect("missing argument")
                    .into(),
                velocity_texture.texel_size().0,
                velocity_texture.texel_size().1,
            );
            self.velocity_texel_size = velocity_texture.texel_size();
        }

        if self.target_texel_size != target_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_target_texel_size")
                    .expect("missing argument")
                    .into(),
                target_texture.texel_size().0,
                target_texture.texel_size().1,
            );
            self.target_texel_size = target_texture.texel_size();
        }
    }
}

struct ImpulsePipeline {
    program: Program,
    radius: f32,
    scale: f32,
    location: (f32, f32),
    direction: (f32, f32),
}

impl ImpulsePipeline {
    const VELOCITY: i32 = 0;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> Self {
        context.use_program(Some(program.program()));
        context.uniform1i(program.uniforms().get("u_velocity"), Self::VELOCITY);
        ImpulsePipeline {
            program,
            radius: 0.0,
            scale: 0.0,
            location: (0.0, 0.0),
            direction: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        location: (f32, f32),
        direction: (f32, f32),
        radius: f32,
        scale: f32,
        target_texture: &BufferedTexture,
    ) -> () {
        context.use_program(Some(self.program.program()));
        target_texture.attach(Self::VELOCITY);
        if self.location != location {
            context.uniform2f(
                self.program.uniforms().get("u_location").unwrap().into(),
                location.0,
                location.1,
            );
            self.location = location;
        }

        if self.direction != direction {
            context.uniform2f(
                self.program.uniforms().get("u_direction").unwrap().into(),
                direction.0,
                direction.1,
            );
            self.direction = direction;
        }

        if self.radius != radius {
            context.uniform1f(
                self.program.uniforms().get("u_radius").unwrap().into(),
                radius,
            );
            self.radius = radius;
        }

        if self.scale != scale {
            context.uniform1f(
                self.program.uniforms().get("u_scale").unwrap().into(),
                scale,
            );
            self.scale = scale;
        }
    }
}

struct DivergencePipeline {
    program: Program,
    texel_size: (f32, f32),
}

impl DivergencePipeline {
    const VELOCITY: i32 = 0;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> Self {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program.uniforms().get("u_velocity").unwrap().into(),
            Self::VELOCITY,
        );
        DivergencePipeline {
            program,
            texel_size: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        velocity_texture: &BufferedTexture,
    ) {
        context.use_program(Some(self.program.program()));
        velocity_texture.attach(Self::VELOCITY);
        if self.texel_size != velocity_texture.texel_size() {
            context.uniform2f(
                self.program.uniforms().get("u_texel_size").unwrap().into(),
                velocity_texture.texel_size().0,
                velocity_texture.texel_size().1,
            );
            self.texel_size = velocity_texture.texel_size();
        }
    }
}

struct GradientSubtractPipeline {
    program: Program,
    texel_size: (f32, f32),
}

impl GradientSubtractPipeline {
    const VELOCITY: i32 = 0;
    const PRESSURE: i32 = 1;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> Self {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program.uniforms().get("u_velocity").unwrap().into(),
            Self::VELOCITY,
        );
        context.uniform1i(
            program.uniforms().get("u_pressure").unwrap().into(),
            Self::PRESSURE,
        );
        GradientSubtractPipeline {
            program,
            texel_size: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        velocity_texture: &BufferedTexture,
        pressure_texture: &BufferedTexture,
    ) -> () {
        debug_assert!(velocity_texture.texel_size() == pressure_texture.texel_size());
        context.use_program(Some(self.program.program()));
        velocity_texture.attach(Self::VELOCITY);
        pressure_texture.attach(Self::PRESSURE);
        if self.texel_size != velocity_texture.texel_size() {
            context.uniform2f(
                self.program.uniforms().get("u_texel_size").unwrap().into(),
                velocity_texture.texel_size().0,
                velocity_texture.texel_size().1,
            );
            self.texel_size = velocity_texture.texel_size();
        }
    }
}

struct JacobiPipeline {
    program: Program,
    alpha: (f32, f32),
    r_beta: (f32, f32),
    texel_size: (f32, f32),
}

impl JacobiPipeline {
    const INITIAL: i32 = 0;
    const SOLUTION: i32 = 1;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> Self {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program.uniforms().get("u_initial").unwrap().into(),
            Self::INITIAL,
        );
        context.uniform1i(
            program.uniforms().get("u_solution").unwrap().into(),
            Self::SOLUTION,
        );
        JacobiPipeline {
            program,
            alpha: (0.0, 0.0),
            r_beta: (0.0, 0.0),
            texel_size: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        alpha: (f32, f32),
        r_beta: (f32, f32),
        initial_texture: &BufferedTexture,
        solution_texutre: &BufferedTexture,
    ) -> () {
        debug_assert!(initial_texture.texel_size() == solution_texutre.texel_size());
        initial_texture.attach(Self::INITIAL);
        solution_texutre.attach(Self::SOLUTION);
        context.use_program(Some(self.program.program()));
        if self.alpha != alpha {
            context.uniform2f(
                self.program.uniforms().get("u_alpha").unwrap().into(),
                alpha.0,
                alpha.1,
            );
            self.alpha = alpha;
        }

        if self.r_beta != r_beta {
            context.uniform2f(
                self.program.uniforms().get("u_r_beta").unwrap().into(),
                r_beta.0,
                r_beta.1,
            );
            self.r_beta = r_beta;
        }

        if self.texel_size != initial_texture.texel_size() {
            context.uniform2f(
                self.program.uniforms().get("u_texel_size").unwrap().into(),
                initial_texture.texel_size().0,
                initial_texture.texel_size().1,
            );
            self.texel_size = initial_texture.texel_size();
        }
    }
}

fn canvas_fill(context: WebGl2RenderingContext, mouse: Rc<UseMouseReturn>) {
    context.get_extension("EXT_color_buffer_float").unwrap();
    context.get_extension("EXT_color_buffer_float").unwrap();
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

    let sim_w = 256;
    let sim_h = 256;
    let dye_w = context.drawing_buffer_width() as usize;
    let dye_h = context.drawing_buffer_height() as usize;
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

    let quad = Quad::create(&context);

    *g.borrow_mut() = Some(Closure::new(move || {
        // Velocity Boundary
        boundary_pipeline.set_arguments(&context, -1.0, &boundary_texture, velocity_texture.read());
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Advect Velocity
        advect_pipeline.set_arguments(
            &context,
            timestep,
            velocity_texture.read(),
            velocity_texture.read(),
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Advect Dye
        advect_pipeline.set_arguments(
            &context,
            timestep,
            velocity_texture.read(),
            dye_texture.read(),
        );
        quad.blit(Some(dye_texture.write()));
        dye_texture.swap();

        // Add impulse
        let cur_mouse: (f32, f32) = (
            mouse.x.get_untracked().div(dye_w as f64) as f32,
            1.0.sub(mouse.y.get_untracked().div(dye_h as f64)) as f32,
        );
        impulse_pipeline.set_arguments(
            &context,
            cur_mouse,
            (cur_mouse.0 - prev_mouse.0, cur_mouse.1 - prev_mouse.1),
            force_radius,
            force_scale,
            velocity_texture.read(),
        );
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();
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
                diffusion_alpha,
                diffusion_beta,
                &temp_texture,
                velocity_texture.read(),
            );
            quad.blit(Some(velocity_texture.write()));
            velocity_texture.swap();
        }

        // Compute Divergance
        divergence_pipeline.set_arguments(&context, velocity_texture.read());
        quad.blit(Some(&temp_texture));

        // Compute Pressure
        pressure_texture
            .read()
            .copy_from(&blank_texture)
            .expect("failed to clear pressure");

        for _ in 0..40 {
            boundary_pipeline.set_arguments(
                &context,
                1.0,
                &boundary_texture,
                pressure_texture.read(),
            );
            quad.blit(Some(pressure_texture.write()));
            pressure_texture.swap();

            jacobi_pipeline.set_arguments(
                &context,
                pressure_alpha,
                pressure_beta,
                &temp_texture,
                pressure_texture.read(),
            );
            quad.blit(Some(pressure_texture.write()));
            pressure_texture.swap();
        }

        // Reapply Boundaries
        boundary_pipeline.set_arguments(&context, -1.0, &boundary_texture, velocity_texture.read());
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // Gradient Subtraction
        gradient_pipeline.set_arguments(&context, velocity_texture.read(), pressure_texture.read());
        quad.blit(Some(velocity_texture.write()));
        velocity_texture.swap();

        // draw the dye
        context.use_program(Some(quad_program.program()));
        context.uniform1i(
            quad_program.uniforms().get("u_texture").unwrap().into(),
            dye_texture.read().attach(0),
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
