use std::cell::RefCell;
use std::convert::Infallible;
use std::f64::consts::PI;
use std::ops::{Div, Mul};
use std::rc::Rc;

use super::common::*;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use web_sys::HtmlElement;
use web_sys::MouseEvent;
use web_sys::Touch;

use web_sys::console;

use leptos_use::{
    UseMouseCoordType, UseMouseEventExtractor, UseMouseOptions, UseMouseReturn, core::Position,
    use_mouse_with_options,
};

use web_sys::WebGl2RenderingContext;

type GL = WebGl2RenderingContext;

#[derive(Clone)]
struct OffsetExtractor;
impl UseMouseEventExtractor for OffsetExtractor {
    fn extract_mouse_coords(&self, event: &MouseEvent) -> Option<(f64, f64)> {
        match event.buttons() % 2 {
            1 => Some((event.offset_x() as f64, event.offset_y() as f64)),
            _ => Some((-1.0, -1.0)),
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

struct AvalanchePipeline {
    program: Program,
    texel_size: (f32, f32),
    max_height: f32,
}

impl AvalanchePipeline {
    const SAND: i32 = 0;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> AvalanchePipeline {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program
                .uniforms()
                .get("u_sand")
                .expect("missing argument")
                .into(),
            Self::SAND,
        );
        AvalanchePipeline {
            program,
            texel_size: (0.0, 0.0),
            max_height: 0.0,
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        sand_texture: &BufferedTexture,
        max_height: f32,
    ) -> () {
        debug_assert!(sand_texture.texel_size() == sand_texture.texel_size());
        context.use_program(Some(self.program.program()));
        sand_texture.attach(Self::SAND);
        if self.max_height != max_height {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_max_height")
                    .expect("missing argument")
                    .into(),
                max_height,
            );
            self.max_height = max_height;
        }
        if self.texel_size != sand_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_texel_size")
                    .expect("missing argument")
                    .into(),
                sand_texture.texel_size().0,
                sand_texture.texel_size().1,
            );
            self.texel_size = sand_texture.texel_size();
        }
    }
}

struct DropPipeline {
    program: Program,
    texel_size: (f32, f32),
    max_height: f32,
    radius: f32,
    center: (f32, f32),
}

impl DropPipeline {
    const SAND: i32 = 0;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> DropPipeline {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program
                .uniforms()
                .get("u_sand")
                .expect("missing argument")
                .into(),
            Self::SAND,
        );
        DropPipeline {
            program,
            texel_size: (0.0, 0.0),
            max_height: 0.0,
            radius: 0.0,
            center: (0.0, 0.0),
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        sand_texture: &BufferedTexture,
        max_height: f32,
        radius: f32,
        center: (f32, f32),
    ) -> () {
        debug_assert!(sand_texture.texel_size() == sand_texture.texel_size());
        context.use_program(Some(self.program.program()));
        sand_texture.attach(Self::SAND);
        if self.max_height != max_height {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_max_height")
                    .expect("missing argument")
                    .into(),
                max_height,
            );
            self.max_height = max_height;
        }
        if self.radius != radius {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_radius")
                    .expect("missing argument")
                    .into(),
                radius,
            );
            self.radius = radius;
        }
        if self.texel_size != sand_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_texel_size")
                    .expect("missing argument")
                    .into(),
                sand_texture.texel_size().0,
                sand_texture.texel_size().1,
            );
            self.texel_size = sand_texture.texel_size();
        }
        if self.center != center {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_center")
                    .expect("missing argument")
                    .into(),
                center.0,
                center.1,
            );
            self.center = center;
        }
    }
}

struct ShadowPipeline {
    program: Program,
    texel_size: (f32, f32),
    direction: (f32, f32),
    tan_theta: f32,
    max_height: f32,
}

impl ShadowPipeline {
    const SAND: i32 = 0;
    pub fn create(context: &WebGl2RenderingContext, program: Program) -> ShadowPipeline {
        context.use_program(Some(program.program()));
        context.uniform1i(
            program
                .uniforms()
                .get("u_sand")
                .expect("missing argument")
                .into(),
            Self::SAND,
        );
        ShadowPipeline {
            program,
            texel_size: (0.0, 0.0),
            direction: (0.0, 0.0),
            tan_theta: 0.0,
            max_height: 0.0,
        }
    }
    pub fn set_arguments(
        &mut self,
        context: &WebGl2RenderingContext,
        sand_texture: &BufferedTexture,
        direction: (f32, f32),
        tan_theta: f32,
        max_height: f32,
    ) -> () {
        debug_assert!(sand_texture.texel_size() == sand_texture.texel_size());
        context.use_program(Some(self.program.program()));
        sand_texture.attach(Self::SAND);
        if self.max_height != max_height {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_max_height")
                    .expect("missing argument")
                    .into(),
                max_height,
            );
            self.max_height = max_height;
        }
        if self.tan_theta != tan_theta {
            context.uniform1f(
                self.program
                    .uniforms()
                    .get("u_tan_theta")
                    .expect("missing argument")
                    .into(),
                tan_theta,
            );
            self.tan_theta = tan_theta;
        }
        if self.texel_size != sand_texture.texel_size() {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_texel_size")
                    .expect("missing argument")
                    .into(),
                sand_texture.texel_size().0,
                sand_texture.texel_size().1,
            );
            self.texel_size = sand_texture.texel_size();
        }
        if self.direction != direction {
            context.uniform2f(
                self.program
                    .uniforms()
                    .get("u_direction")
                    .expect("missing argument")
                    .into(),
                direction.0,
                direction.1,
            );
            self.direction = direction;
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let mouse = use_mouse_with_options(
        UseMouseOptions::default()
            .target(canvas_ref)
            .initial_value(Position { x: -1.0, y: -1.0 })
            .reset_on_touch_ends(true)
            .coord_type(UseMouseCoordType::Custom(OffsetExtractor)),
    );
    let mouse_rc = Rc::new(mouse);
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
            canvas_fill(context.clone(), mouse_rc.clone(), count.into());
        }
    });

    view! {
    <button
        on:click=move |_| *set_count.write() += 1
    >
        {move || {if count.get() % 2 == 0 {"STOP"} else {"START"}}}
    </button> <canvas node_ref=canvas_ref /> }
}

fn canvas_fill(context: WebGl2RenderingContext, mouse: Rc<UseMouseReturn>, count: Signal<i32>) {
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

    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let avalanche_program = Program::create(&context, &quad_vert_shader, &avalanche_frag_shader);
    let shadow_program = Program::create(&context, &quad_vert_shader, &shadow_frag_shader);
    let drop_program = Program::create(&context, &quad_vert_shader, &drop_frag_shader);

    let mut avalanche_pipeline = AvalanchePipeline::create(&context, avalanche_program);
    let mut shadow_pipeline = ShadowPipeline::create(&context, shadow_program);
    let mut drop_pipeline = DropPipeline::create(&context, drop_program);

    let mut sand = make_sand(&context, window_w, window_h);

    let (next_fame, set_next_frame) = signal(());

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    let mut prev_time = None::<f64>;
    let mut angle = 0.0;

    let quad = Quad::create(&context);

    Effect::new(move || {
        next_fame.get();
        let now = window().performance().unwrap().now();
        if !prev_time.is_some() || now - prev_time.unwrap() > 16.0 {
            prev_time = Some(now);
            avalanche_pipeline.set_arguments(&context, sand.read(), 255.0);
            quad.blit(Some(&sand.write()));
            sand.swap();

            let cur_mouse: (f32, f32) = (
                mouse.x.get_untracked().div(window_w as f64) as f32,
                1.0 - mouse.y.get_untracked().div(window_h as f64) as f32,
            );
            if cur_mouse.0 > 0.0 {
                drop_pipeline.set_arguments(&context, sand.read(), 255.0, 10.0, cur_mouse);
                quad.blit(Some(&sand.write()));
                sand.swap();
                console::log_1(&"Sanding".into());
            }
        }
        if count.get() % 2 == 0 {
            angle = now % 20000.0 * (PI / 10000.0);
        }

        shadow_pipeline.set_arguments(
            &context,
            sand.read(),
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

fn make_sand(context: &WebGl2RenderingContext, width: usize, height: usize ) -> SwappableTexture {
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
