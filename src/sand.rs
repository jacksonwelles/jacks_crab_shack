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

use leptos_use::UseEventListenerOptions;
use leptos_use::signal_throttled;
use leptos_use::use_event_listener_with_options;

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

render_pipeline!(RandomPipeline, "shaders/random.frag");

render_pipeline!(WindPipeline, "shaders/wind.frag");

render_pipeline!(DrawPipeline, "shaders/draw.frag");


struct RandomStage {
    context: WebGl2RenderingContext,
    rand: Rc<BufferedTexture>,
    quad: Rc<Quad>,
    pipeline: RandomPipeline,
}

impl RandomStage {
    pub fn update(&mut self) -> () {
        self.pipeline
            .set_arguments(&self.context, window().performance().unwrap().now() as f32);
        self.quad.blit(Some(&self.rand));
    }
}
struct AvalancheStage {
    context: WebGl2RenderingContext,
    sand: Rc<RefCell<SwappableTexture>>,
    diff: SwappableTexture,
    rand: Rc<BufferedTexture>,
    quad: Rc<Quad>,
    calc: AvalancheCalcPipeline,
    apply: AvalancheApplyPipeline,
    max_height: f32,
}

impl AvalancheStage {
    pub fn update(&mut self) -> () {
        self.calc.set_arguments(
            &self.context,
            self.max_height,
            self.sand.borrow().read(),
            &self.rand,
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
    sand: Rc<RefCell<SwappableTexture>>,
    quad: Rc<Quad>,
    lookahead: Rc<RefCell<SwappableTexture>>,
    pipeline: LookaheadPipeline,
    scale: f32,
}

impl LookaheadStage {
    pub fn update(&mut self, direction: (f32, f32)) -> () {
        self.pipeline.set_arguments(
            &self.context,
            &self.sand.borrow().read(),
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

struct ShadowStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    sand: Rc<RefCell<SwappableTexture>>,
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
            &self.sand.borrow().read(),
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

struct WindStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    sand: Rc<RefCell<SwappableTexture>>,
    shadow: Rc<BufferedTexture>,
    random: Rc<BufferedTexture>,
    pipeline: WindPipeline,
    max_height: f32,
    pickup_rate: f32,
    wind_speed: f32,
}

impl WindStage {
    pub fn update(&mut self, direction: (f32, f32)) {
        self.pipeline.set_arguments(
            &self.context,
            direction,
            self.sand.borrow().read().texel_size(),
            self.wind_speed,
            self.max_height,
            self.sand.borrow().read(),
            &self.random,
            &self.shadow,
            self.pickup_rate,
        );
        self.quad.blit(Some(self.sand.borrow().write()));
        self.sand.borrow_mut().swap();
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

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (mouse, set_mouse) = signal((false, 0i32, 0i32));
    let evt_options = UseEventListenerOptions::default().passive(true);
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::mousedown,
        move |evt| {
            *set_mouse.write() = (true, evt.offset_x(), evt.offset_y());
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        window(),
        leptos::ev::mouseup,
        move |_| {
            set_mouse.update(|tup| tup.0 = false);
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::mousemove,
        move |evt| {
            set_mouse.update(|tup| {
                tup.1 = evt.offset_x();
                tup.2 = evt.offset_y();
            });
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::touchstart,
        move |evt| {
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
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::touchend,
        move |_| set_mouse.update(|tup| tup.0 = false),
        evt_options,
    );
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::touchmove,
        move |evt| {
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
        },
        evt_options,
    );
    let (fps, set_fps) = signal(0.0);
    let (sun_move, set_sun_move) = signal(true);
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
            canvas_fill(
                context.clone(),
                sun_move.into(),
                set_fps.into(),
                mouse.into(),
            );
        }
    });

    let fps_throttled: Signal<f64> = signal_throttled(fps, 500.0);
    view! {
     <canvas style:touch-action="pinch-zoom" node_ref=canvas_ref />
     <br/>
     <button
        on:click=move |_| *set_sun_move.write() = ! sun_move.get()
    >
        {move || {if sun_move.get() {"STOP"} else {"START"}}}
    </button> <br/>
    <pre> {move||{
        format!("{:.2}",fps_throttled.get())
    }}</pre>}
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    sun_move: Signal<bool>,
    set_fps: WriteSignal<f64>,
    mouse: Signal<(bool, i32, i32)>,
) {
    context.get_extension("EXT_color_buffer_float").unwrap();
    context.get_extension("OES_texture_float_linear").unwrap();
    let quad_vert_shader = compile_shader(
        &context,
        GL::VERTEX_SHADER,
        include_str!("shaders/quad.vert"),
    )
    .unwrap();
    let make_prog = |frag_source: &str| {
        let compiled = compile_shader(&context, GL::FRAGMENT_SHADER, frag_source).unwrap();
        Program::create(&context, &quad_vert_shader, &compiled)
    };

    let window_w = context.drawing_buffer_width() as usize;
    let window_h = context.drawing_buffer_height() as usize;

    let sand_w = window_w;
    let sand_h = window_h;
    let scale = 4.0f32;
    let max_height = 255.0f32;
    let sun_angle = 38.0f32;
    let radius = 200.0;
    let drop_period = 16.0;
    let wind_speed = 0.001;
    let pickup_rate = 0.1;
    let frame_period = 8.333;

    let avalanche_calc_pipeline = AvalancheCalcPipeline::create(
        &context,
        make_prog(include_str!("shaders/avalanche_calc.frag")),
    );
    let avalanche_apply_pipeline = AvalancheApplyPipeline::create(
        &context,
        make_prog(include_str!("shaders/avalanche_apply.frag")),
    );
    let shadow_pipeline = ShadowPipeline::create(
        &context,
        make_prog(include_str!("shaders/optimized_shadow.frag")),
    );
    let drop_pipeline =
        DropPipeline::create(&context, make_prog(include_str!("shaders/drop_sand.frag")));
    let lookahead_pipeline = LookaheadPipeline::create(
        &context,
        make_prog(include_str!("shaders/precompute_shadow.frag")),
    );
    let random_pipeline =
        RandomPipeline::create(&context, make_prog(include_str!("shaders/random.frag")));
    let draw_pipeline =
        DrawPipeline::create(&context, make_prog(include_str!("shaders/draw.frag")));
    let wind_pipeline =
        WindPipeline::create(&context, make_prog(include_str!("shaders/wind.frag")));

    let shadow = Rc::new(make_shadow(&context, window_w, window_w));
    let random = Rc::new(make_rand(&context, sand_w, sand_h));
    let delta_sand = make_sand(&context, sand_w, sand_h);
    let sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));
    let lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, window_w, window_h,
    )));

    let (next_frame, set_next_frame) = signal(());
    let next_frame_throttled: Signal<()> = signal_throttled(next_frame, frame_period);

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

    let mut avalanche_stage = AvalancheStage {
        context: context.clone(),
        sand: sand.clone(),
        diff: delta_sand,
        rand: random.clone(),
        quad: quad.clone(),
        calc: avalanche_calc_pipeline,
        apply: avalanche_apply_pipeline,
        max_height,
    };

    let mut random_stage = RandomStage {
        context: context.clone(),
        rand: random.clone(),
        quad: quad.clone(),
        pipeline: random_pipeline,
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

    let mut lookahead_stage = LookaheadStage {
        context: context.clone(),
        sand: sand.clone(),
        quad: quad.clone(),
        lookahead: lookahead.clone(),
        pipeline: lookahead_pipeline,
        scale,
    };

    let mut shadow_stage = ShadowStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: shadow.clone(),
        sand: sand.clone(),
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

    let mut wind_stage = WindStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: shadow.clone(),
        sand: sand.clone(),
        pipeline: wind_pipeline,
        random: random.clone(),
        max_height,
        pickup_rate,
        wind_speed,
    };

    Effect::new(move || {
        signal_drop_throttled.get();
        let (active, x, y) = mouse.get_untracked();
        if !active {
            return;
        }
        drop_stage.update(x as f32, y as f32);
    });

    let mut prev_frame = 0.0;
    Effect::new(move || {
        next_frame_throttled.get();
        let direction = (angle.get().cos() as f32, angle.get().sin() as f32);
        let now = window().performance().unwrap().now();
        *set_fps.write() = 1000.0 / (now - prev_frame);
        prev_frame = now;
        avalanche_stage.update();
        lookahead_stage.update(direction);
        shadow_stage.update(direction);
        draw_stage.update(direction);
        wind_stage.update(direction);
        random_stage.update();
    });
}

fn make_shadow(context: &WebGl2RenderingContext, width: usize, height: usize) -> BufferedTexture {
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
fn make_sand(context: &WebGl2RenderingContext, width: usize, height: usize) -> SwappableTexture {
    return SwappableTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RG32F,
        width as i32,
        height as i32,
        0,
        GL::RG,
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

fn make_rand(context: &WebGl2RenderingContext, width: usize, height: usize) -> BufferedTexture {
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
