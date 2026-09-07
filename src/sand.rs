use std::cell::RefCell;
use std::cmp::min;
use std::convert::Infallible;
use std::rc::Rc;

use utility::prelude::*;

use utility_macro::render_pipeline;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use leptos::logging::log;

use leptos_use::UseEventListenerOptions;
use leptos_use::signal_throttled;
use leptos_use::use_event_listener_with_options;

use web_sys::HtmlElement;
use web_sys::WebGl2RenderingContext;

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
    pipeline: Rc<RefCell<LookaheadPipeline>>,
    scale: f32,
}

impl LookaheadStage {
    pub fn update(&mut self, direction: (f32, f32)) -> () {
        self.pipeline.borrow_mut().set_arguments(
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
            self.pipeline.borrow_mut().set_arguments(
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
    pipeline: Rc<RefCell<ShadowPipeline>>,
    scale: f32,
    max_height: f32,
}

impl ShadowStage {
    pub fn update(&mut self, direction: (f32, f32, f32)) {
        self.pipeline.borrow_mut().set_arguments(
            &self.context,
            &self.sand.borrow().read(),
            self.lookahead.borrow().read(),
            self.scale,
            (
                self.shadow.texel_size().0,
                self.shadow.texel_size().1,
                1.0 / self.max_height,
            ),
            direction,
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
}

impl DrawStage {
    pub fn update(&mut self, direction: (f32, f32, f32)) {
        self.pipeline.set_arguments(
            &self.context,
            &self.sand.borrow().read(),
            &self.shadow,
            (
                self.shadow.texel_size().0,
                self.shadow.texel_size().1,
                1.0 / self.max_height,
            ),
            direction,
        );
        self.quad.blit(None);
    }
}


#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    let (mouse, set_mouse) = signal((None::<(i32, i32)>, 0i32, 0i32));
    let evt_options = UseEventListenerOptions::default().passive(true);
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::mousedown,
        move |evt| {
            *set_mouse.write() = (Some((evt.offset_x(), evt.offset_y())), evt.offset_x(), evt.offset_y());
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        window(),
        leptos::ev::mouseup,
        move |_| {
            set_mouse.update(|tup| tup.0 = None);
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
                set_mouse.update(|tup| tup.0 = None);
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
            let canvas_x = touch.client_x() - rect.x() as i32;
            let canvas_y = touch.client_y() - rect.y() as i32;
            *set_mouse.write() = (
                Some((canvas_x, canvas_y)),
                canvas_x,
                canvas_y as i32,
            );
        },
        evt_options,
    );
    let _ = use_event_listener_with_options(
        canvas_ref,
        leptos::ev::touchend,
        move |_| set_mouse.update(|tup| tup.0 = None),
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
    let input_mode = RwSignal::new("sand".to_string());
    let (fps, set_fps) = signal(0.0);
    let (wind_on, set_wind_on) = signal(true);
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
                wind_on.into(),
                set_fps.into(),
                mouse.into(),
                input_mode.into(),
            );
        }
    });

    let fps_throttled: Signal<f64> = signal_throttled(fps, 500.0);
    view! {
        <canvas style:touch-action="pinch-zoom" node_ref=canvas_ref />
        <br />
        <pre>{move || { format!("{:.2}", fps_throttled.get()) }}</pre>
        <button on:click=move |_| {
            *set_wind_on.write() = !wind_on.get();
        }>{move || { if wind_on.get() { "WIND STOP" } else { "WIND START" } }}</button>
        <br />
        <br />
        <fieldset>
            <label>
                "Sand" <input type="radio" name="color" value="sand" bind:group=input_mode />
            </label>
            <label>
                "Wind" <input type="radio" name="color" value="wind" bind:group=input_mode />
            </label>
            <label>
                "Sun" <input type="radio" name="color" value="shadow" bind:group=input_mode />
            </label>
        </fieldset>
    }
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    wind_on: Signal<bool>,
    set_fps: WriteSignal<f64>,
    mouse: Signal<(Option<(i32, i32)>, i32, i32)>,
    input_mode : Signal<String>,
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
    let radius = 100.0;
    let drop_period = 16.0;
    let wind_speed = 0.0005;
    let pickup_rate = 0.5;
    let frame_period = 8.333;

    let avalanche_calc_pipeline = AvalancheCalcPipeline::create(
        &context,
        make_prog(include_str!("shaders/avalanche_calc.frag")),
    );
    let avalanche_apply_pipeline = AvalancheApplyPipeline::create(
        &context,
        make_prog(include_str!("shaders/avalanche_apply.frag")),
    );
    let shadow_pipeline = Rc::new(RefCell::new(ShadowPipeline::create(
        &context,
        make_prog(include_str!("shaders/optimized_shadow.frag")),
    )));
    let drop_pipeline =
        DropPipeline::create(&context, make_prog(include_str!("shaders/drop_sand.frag")));
    let lookahead_pipeline = Rc::new(RefCell::new(LookaheadPipeline::create(
        &context,
        make_prog(include_str!("shaders/precompute_shadow.frag")),
    )));
    let random_pipeline =
        RandomPipeline::create(&context, make_prog(include_str!("shaders/random.frag")));
    let draw_pipeline =
        DrawPipeline::create(&context, make_prog(include_str!("shaders/draw.frag")));
    let wind_pipeline =
        WindPipeline::create(&context, make_prog(include_str!("shaders/wind.frag")));

    let shadow = Rc::new(make_shadow(&context, sand_w, sand_h));
    let random = Rc::new(make_rand(&context, sand_w, sand_h));
    let delta_sand = make_sand(&context, sand_w, sand_h);
    let sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));
    let lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, window_w, window_h,
    )));

    // let wind_shadow = Rc::new(make_shadow(&context, sand_w / 2, sand_h / 2));
    // let wind_shadow_lookahead = Rc::new(RefCell::new(make_shadow_lookahead(&context, sand_w / 2, sand_h / 2)));

    let wind_shadow = Rc::new(make_shadow(&context, sand_w, sand_h));
    let wind_shadow_lookahead = Rc::new(RefCell::new(make_shadow_lookahead(&context, sand_w, sand_h)));

    let (next_frame, set_next_frame) = signal(());
    let next_frame_throttled: Signal<()> = signal_throttled(next_frame, frame_period);

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    let (signal_drop, set_signal_drop) = signal(());
    let signal_drop_throttled: Signal<()> = signal_throttled(signal_drop, drop_period);

    {
        Effect::new(move || {
            next_frame.get();
            if mouse.get_untracked().0.is_some() {
                set_signal_drop.write();
            }
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

    let mut wind_lookahead_stage = LookaheadStage {
        context: context.clone(),
        sand: sand.clone(),
        quad: quad.clone(),
        lookahead: wind_shadow_lookahead.clone(),
        pipeline: lookahead_pipeline.clone(),
        scale,
    };

    let mut wind_shadow_stage = ShadowStage {
        context: context.clone(),
        quad: quad.clone(),
        sand: sand.clone(),
        shadow: wind_shadow.clone(),
        lookahead: wind_shadow_lookahead.clone(),
        pipeline: shadow_pipeline.clone(),
        scale,
        max_height: max_height,
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
    };

    let mut draw_stage = DrawStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: shadow.clone(),
        sand: sand.clone(),
        pipeline: draw_pipeline,
        max_height,
    };

    let mut wind_stage = WindStage {
        context: context.clone(),
        quad: quad.clone(),
        shadow: wind_shadow.clone(),
        sand: sand.clone(),
        pipeline: wind_pipeline,
        random: random.clone(),
        max_height,
        pickup_rate,
        wind_speed,
    };

    let mut wind_dir = (0.0, 0.0, 0.0);
    let mut sun_dir = (20.0, 60.0, 500.0);
    let mut prev_frame = 0.0;
    Effect::new(move || {
        next_frame_throttled.get();
        let (click_start, mouse_x, mouse_y) = mouse.get_untracked();
        if let Some((start_x, start_y)) = click_start {
            let dir = ((mouse_x - start_x) as f32 , (mouse_y - start_y) as f32);
            match input_mode.get_untracked().as_str() {
                "shadow" => {
                    sun_dir = (dir.0, dir.1, min(window_h, window_w) as f32 / 4.0);
                    // log!("sun direction {:?}", sun_dir)
                },

                "wind" => {
                    let mag = (dir.0.powi(2) + dir.1.powi(2)).sqrt();
                    wind_dir = (dir.0 / mag, dir.1 / mag, 38.0f32.to_radians().tan());
                },

                "sand" => {
                    drop_stage.update(mouse_x as f32, mouse_y as f32);
                }
                _ => ()
            };
        }
        let now = window().performance().unwrap().now();
        *set_fps.write() = 1000.0 / (now - prev_frame);
        prev_frame = now;
        avalanche_stage.update();
        lookahead_stage.update((sun_dir.0, sun_dir.1));
        shadow_stage.update(sun_dir);
        wind_lookahead_stage.update((wind_dir.0, wind_dir.1));
        wind_shadow_stage.update(wind_dir);
        draw_stage.update(sun_dir);
        // if wind_on.get_untracked() {
        //     wind_stage.update((wind_dir.0, wind_dir.1));
        // }
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
