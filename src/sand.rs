use std::cell::Cell;
use std::cell::RefCell;
use std::convert::Infallible;
use std::f64::consts::PI;
use std::rc::Rc;

use utility::prelude::*;

use utility_macro::render_pipeline;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use leptos_use::signal_throttled;
use leptos_use::use_event_listener;

use web_sys::HtmlElement;
use web_sys::WebGl2RenderingContext;
use web_sys::console;

type GL = WebGl2RenderingContext;

render_pipeline!(QuadPipeline, "shaders/quad.frag");
render_pipeline!(AvalancheCalcPipeline, "shaders/avalanche_calc.frag");

render_pipeline!(AvalancheApplyPipeline, "shaders/avalanche_apply.frag");

render_pipeline!(DropPipeline, "shaders/drop_sand.frag");

render_pipeline!(ShadowPipeline, "shaders/optimized_shadow.frag");

render_pipeline!(LookaheadPipeline, "shaders/precompute_shadow.frag");

render_pipeline!(ShiftPipeline, "shaders/shift_rand.frag");

render_pipeline!(SmoothPipeline, "shaders/smooth.frag");

render_pipeline!(DrawPipeline, "shaders/draw.frag");

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (mouse, set_mouse) = signal((false, 0i32, 0i32));
    let _ = use_event_listener(canvas_ref, leptos::ev::mousedown, move |evt| {
        *set_mouse.write() = (true, evt.offset_x(), evt.offset_y());
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::mouseup, move |_| {
        set_mouse.update(|tup| tup.0 = false);
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::mousemove, move |evt| {
        set_mouse.update(|tup| {
            tup.1 = evt.offset_x();
            tup.2 = evt.offset_y();
        });
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::touchstart, move |evt| {
        if evt.touches().length() != 1 {
            set_mouse.update(|tup| tup.0 = false);
            return;
        }
        let touch = evt.touches().item(0).unwrap();
        let element = touch
            .target()
            .unwrap()
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .clone();
        let rect = element.get_bounding_client_rect();
        *set_mouse.write() = (
            true,
            touch.client_x() - rect.x() as i32,
            touch.client_y() - rect.y() as i32,
        );
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::touchend, move |_| {
        set_mouse.update(|tup| tup.0 = false)
    });
    let _ = use_event_listener(canvas_ref, leptos::ev::touchmove, move |evt| {
        let touch = evt.touches().item(0).unwrap();
        let element = touch
            .target()
            .unwrap()
            .dyn_ref::<HtmlElement>()
            .unwrap()
            .clone();
        let rect = element.get_bounding_client_rect();
        set_mouse.update(|tup| {
            tup.1 = touch.client_x() - rect.x() as i32;
            tup.2 = touch.client_y() - rect.y() as i32;
        });
    });
    let (sun_move, set_sun_move) = signal(true);
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
     <canvas style:touch-action="pinch-zoom" node_ref=canvas_ref />
     <br/>
     <button
        on:click=move |_| *set_sun_move.write() = ! sun_move.get()
    >
        {move || {if sun_move.get() {"STOP"} else {"START"}}}
    </button> }
}

struct Shift {
    context: WebGl2RenderingContext,
    rand: Rc<RefCell<SwappableTexture>>,
    quad: Rc<Quad>,
    shift: ShiftPipeline,
    tick: usize,
}

impl Shift {
    pub fn update(&mut self) -> () {
        let direction = if self.tick % self.rand.borrow().read().width() as usize == 0 {
            (1.0, 1.0)
        } else {
            (1.0, 0.0)
        };
        self.tick += 1;

        self.shift.set_arguments(
            &self.context,
            self.rand.borrow().read(),
            self.rand.borrow().read().texel_size(),
            direction,
        );
        self.quad.blit(Some(self.rand.borrow().write()));
        self.rand.borrow_mut().swap();
    }
}
struct Avalanche {
    context: WebGl2RenderingContext,
    sand: Rc<RefCell<SwappableTexture>>,
    diff: SwappableTexture,
    rand: Rc<RefCell<SwappableTexture>>,
    quad: Rc<Quad>,
    calc: AvalancheCalcPipeline,
    apply: AvalancheApplyPipeline,
    max_height: f32,
}

impl Avalanche {
    pub fn update(&mut self) -> () {
        self.calc.set_arguments(
            &self.context,
            self.max_height,
            self.sand.borrow().read(),
            self.rand.borrow().read(),
            self.sand.borrow().read().texel_size(),
        );
        self.quad.blit(Some(self.diff.write()));
        self.diff.swap();

        self.apply.set_arguments(
            &self.context,
            self.max_height,
            self.sand.borrow().read(),
            self.diff.read(),
            self.sand.borrow().read().texel_size(),
        );

        self.quad.blit(Some(self.sand.borrow().write()));
        self.sand.borrow_mut().swap();
    }
}

struct DropStage {
    context: WebGl2RenderingContext,
    sand: Rc<RefCell<SwappableTexture>>,
    quad: Rc<Quad>,
    drop: DropPipeline,
    window_w: usize,
    window_h: usize,
    radius: f32,
    max_height: f32,
}

impl DropStage {
    pub fn update(&mut self, x: f32, y: f32) -> () {
        let pos: (f32, f32) = (x / self.window_w as f32, 1.0 - y / self.window_h as f32);
        self.drop.set_arguments(
            &self.context,
            self.sand.borrow().read(),
            self.sand.borrow().read().texel_size(),
            self.max_height,
            self.radius,
            pos,
        );
        self.quad.blit(Some(&self.sand.borrow().write()));
        self.sand.borrow_mut().swap();
    }
}

struct LookaheadStage {
    context: WebGl2RenderingContext,
    smooth_sand: Rc<BufferedTexture>,
    quad: Rc<Quad>,
    lookahead: Rc<RefCell<SwappableTexture>>,
    pipeline: LookaheadPipeline,
    scale: f32,
}

impl LookaheadStage {
    pub fn update(&mut self, direction: (f32, f32)) -> () {
        self.pipeline.set_arguments(
            &self.context,
            &self.smooth_sand,
            self.scale,
            direction,
            self.lookahead.borrow().read().texel_size(),
            0.0,
        );

        self.quad.blit(Some(self.lookahead.borrow().write()));
        self.lookahead.borrow_mut().swap();
        for i in 1..4 {
            self.pipeline.set_arguments(
                &self.context,
                self.lookahead.borrow().read(),
                self.scale,
                direction,
                self.lookahead.borrow().read().texel_size(),
                i as f32,
            );
            self.quad.blit(Some(self.lookahead.borrow().write()));
            self.lookahead.borrow_mut().swap();
        }
    }
}

struct SmoothStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    sand: Rc<RefCell<SwappableTexture>>,
    smooth_sand: Rc<BufferedTexture>,
    pipeline: SmoothPipeline,
}

impl SmoothStage {
    pub fn update(&mut self) -> () {
        self.pipeline.set_arguments(
            &self.context,
            self.sand.borrow().read(),
            self.sand.borrow().read().texel_size(),
        );
        self.quad.blit(Some(&self.smooth_sand));
    }
}

struct ShadowStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    smooth_sand: Rc<BufferedTexture>,
    shadow: Rc<BufferedTexture>,
    lookahead: Rc<RefCell<SwappableTexture>>,
    pipeline: ShadowPipeline,
    scale: f32,
    max_height: f32,
    sun_angle: f32,
}

impl ShadowStage {
    pub fn update(&mut self, direction: (f32, f32)) {
        self.pipeline.set_arguments(
            &self.context,
            &self.smooth_sand,
            self.lookahead.borrow().read(),
            self.scale,
            (
                self.shadow.texel_size().0,
                self.shadow.texel_size().1,
                1.0 / self.max_height,
            ),
            (direction.0, direction.1, self.sun_angle.to_radians().tan()),
        );
        self.quad.blit(Some(&self.shadow));
    }
}

struct DrawStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    sand: Rc<RefCell<SwappableTexture>>,
    shadow: Rc<BufferedTexture>,
    pipeline: DrawPipeline,
    max_height: f32,
    sun_angle: f32,
}

impl DrawStage {
    pub fn update(&mut self, direction: (f32, f32)) {
        self.pipeline.set_arguments(
            &self.context,
            &self.sand.borrow().read(),
            &self.shadow,
            (
                self.shadow.texel_size().0,
                self.shadow.texel_size().1,
                1.0 / self.max_height,
            ),
            (direction.0, direction.1, self.sun_angle.to_radians().tan()),
        );
        self.quad.blit(None);
    }
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    sun_move: Signal<bool>,
    mouse: Signal<(bool, i32, i32)>,
) {
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

    let avalanche_calc_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/avalanche_calc.frag"),
    )
    .unwrap();

    let avalanche_apply_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/avalanche_apply.frag"),
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

    let shift_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/shift_rand.frag"),
    )
    .unwrap();

    let smooth_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/smooth.frag"),
    )
    .unwrap();

    let draw_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        include_str!("shaders/draw.frag"),
    )
    .unwrap();

    let window_w = context.drawing_buffer_width() as usize;
    let window_h = context.drawing_buffer_height() as usize;

    let sand_w = window_w;
    let sand_h = window_h;
    let scale = 4.0f32;
    let max_height = 255.0f32;
    let sun_angle = 46.0f32;
    let radius = 20.0;
    let drop_period = 16.0;

    let avalanche_calc_program =
        Program::create(&context, &quad_vert_shader, &avalanche_calc_frag_shader);
    let avalanche_apply_program =
        Program::create(&context, &quad_vert_shader, &avalanche_apply_frag_shader);
    let shadow_program = Program::create(&context, &quad_vert_shader, &shadow_frag_shader);
    let drop_program = Program::create(&context, &quad_vert_shader, &drop_frag_shader);
    let lookahead_program = Program::create(&context, &quad_vert_shader, &lookahead_frag_shader);
    let shift_program = Program::create(&context, &quad_vert_shader, &shift_frag_shader);
    let smooth_program = Program::create(&context, &quad_vert_shader, &smooth_frag_shader);
    let draw_program = Program::create(&context, &quad_vert_shader, &draw_frag_shader);

    let avalanche_calc_pipeline = AvalancheCalcPipeline::create(&context, avalanche_calc_program);
    let avalanche_apply_pipeline =
        AvalancheApplyPipeline::create(&context, avalanche_apply_program);
    let shadow_pipeline = ShadowPipeline::create(&context, shadow_program);
    let drop_pipeline = DropPipeline::create(&context, drop_program);
    let lookahead_pipeline = LookaheadPipeline::create(&context, lookahead_program);
    let shift_pipeline = ShiftPipeline::create(&context, shift_program);
    let smooth_pipeline = SmoothPipeline::create(&context, smooth_program);
    let draw_pipeline = DrawPipeline::create(&context, draw_program);

    let shadow = Rc::new(make_shadow(&context, window_w, window_w));
    let random = Rc::new(RefCell::new(make_avalance_rand(&context, sand_w, sand_h)));
    let delta_sand = make_sand(&context, sand_w, sand_h);
    let sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));
    let smooth_sand = Rc::new(make_smooth_sand(&context, sand_w, sand_h));
    let lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, window_w, window_h,
    )));

    let (next_frame, set_next_frame) = signal(());

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    let (signal_drop, set_signal_drop) = signal(());
    let signal_drop_throttled: Signal<()> = signal_throttled(signal_drop, drop_period);
    let angle = Rc::new(Cell::new(0.0));

    let mut prev_time = None::<f64>;

    {
        let angle = angle.clone();
        Effect::new(move || {
            next_frame.get();
            let now = window().performance().unwrap().now();
            if sun_move.get_untracked() {
                if prev_time.is_some() {
                    angle.update(|x| x + (now - prev_time.unwrap()) % 20000.0 * (PI / 10000.0));
                }
            }
            if mouse.get_untracked().0 {
                set_signal_drop.write();
            }
            prev_time = Some(now);
            request_animation_frame(move || {
                *set_next_frame.write();
            });
        });
    }

    let quad = Rc::new(Quad::create(&context));

    let mut avalanche_stage = Avalanche {
        context: context.clone(),
        sand: sand.clone(),
        diff: delta_sand,
        rand: random.clone(),
        quad: quad.clone(),
        calc: avalanche_calc_pipeline,
        apply: avalanche_apply_pipeline,
        max_height,
    };

    let mut shift_stage = Shift {
        context: context.clone(),
        rand: random.clone(),
        quad: quad.clone(),
        shift: shift_pipeline,
        tick: 0,
    };

    let mut drop_stage = DropStage {
        context: context.clone(),
        sand: sand.clone(),
        quad: quad.clone(),
        drop: drop_pipeline,
        window_w,
        window_h,
        radius,
        max_height,
    };

    let mut smooth_stage = SmoothStage {
        context: context.clone(),
        quad: quad.clone(),
        sand: sand.clone(),
        smooth_sand: smooth_sand.clone(),
        pipeline: smooth_pipeline,
    };

    let mut lookahead_stage = LookaheadStage {
        context: context.clone(),
        smooth_sand: smooth_sand.clone(),
        quad: quad.clone(),
        lookahead: lookahead.clone(),
        pipeline: lookahead_pipeline,
        scale,
    };

    let mut shadow_stage = ShadowStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: shadow.clone(),
        smooth_sand: smooth_sand.clone(),
        lookahead: lookahead.clone(),
        pipeline: shadow_pipeline,
        scale,
        max_height,
        sun_angle,
    };

    let mut draw_stage = DrawStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: shadow.clone(),
        sand: sand.clone(),
        pipeline: draw_pipeline,
        max_height,
        sun_angle,
    };

    Effect::new(move || {
        signal_drop_throttled.get();
        let (_, x, y) = mouse.get_untracked();
        drop_stage.update(x as f32, y as f32);
    });

    Effect::new(move || {
        next_frame.get();
        let direction = (angle.get().cos() as f32, angle.get().sin() as f32);

        avalanche_stage.update();
        smooth_stage.update();
        lookahead_stage.update(direction);
        shadow_stage.update(direction);
        draw_stage.update(direction);
        shift_stage.update();
    });
}

fn make_shadow(context: &WebGl2RenderingContext, width: usize, height: usize) -> BufferedTexture {
    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::R32F,
        width as i32,
        height as i32,
        0,
        GL::RED,
        GL::FLOAT,
        None::<Infallible>,
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
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

struct ShiftState {
    x: u8,
    y: u8,
    z: u8,
    a: u8,
}

fn xshift(state: &mut ShiftState) -> u8 {
    let t = state.x ^ (state.x << 5);
    state.x = state.y;
    state.y = state.z;
    state.z = state.a;
    state.a = state.z ^ (state.z >> 1) ^ t ^ (t << 3);
    state.a
}

fn make_avalance_rand(
    context: &WebGl2RenderingContext,
    width: usize,
    height: usize,
) -> SwappableTexture {
    let mut state = ShiftState {
        x: 0,
        y: 0,
        z: 0,
        a: 1,
    };
    for _ in 0..20 {
        xshift(&mut state);
    }
    let data: Vec<u8> = (0..(width * height)).map(|_| xshift(&mut state)).collect();

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
        Some(ArrayView::create(&data)),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
}

fn make_smooth_sand(
    context: &WebGl2RenderingContext,
    width: usize,
    height: usize,
) -> BufferedTexture {
    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::R16F,
        width as i32,
        height as i32,
        0,
        GL::RED,
        GL::HALF_FLOAT,
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
        GL::RGBA16F,
        width as i32,
        height as i32,
        0,
        GL::RGBA,
        GL::HALF_FLOAT,
        None::<Infallible>,
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
}
