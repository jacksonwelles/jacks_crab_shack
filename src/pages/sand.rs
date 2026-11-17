use std::cell::RefCell;
use std::convert::Infallible;
use std::ops::Mul;
use std::rc::Rc;

use utility::prelude::*;

use utility_macro::render_pipeline;

use leptos::html::Canvas;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

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

render_pipeline!(SunDialPipeline, "shaders/sun_dial.frag");

render_pipeline!(WindDialPipeline, "shaders/wind_dial.frag");

struct WindDialStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    pipeline: WindDialPipeline,
}

impl WindDialStage {
    pub fn update(&mut self, sun_location: (f32, f32)) -> () {
        self.pipeline.set_arguments(&self.context, sun_location);
        self.quad.blit(None);
    }
}

struct SunDialStage {
    context: WebGl2RenderingContext,
    quad: Rc<Quad>,
    pipeline: SunDialPipeline,
}

impl SunDialStage {
    pub fn update(&mut self, sun_location: (f32, f32)) -> () {
        self.pipeline.set_arguments(&self.context, sun_location);
        self.quad.blit(None);
    }
}
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
    radius: f32,
    max_height: f32,
}

impl DropStage {
    pub fn update(&mut self, x: f32, y: f32) -> () {
        let pos: (f32, f32) = (x, y);
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
}

impl WindStage {
    pub fn update(&mut self, direction: (f32, f32), speed: f32, pickup_rate: f32) {
        self.pipeline.set_arguments(
            &self.context,
            direction,
            self.sand.borrow().read().texel_size(),
            speed,
            self.max_height,
            self.sand.borrow().read(),
            &self.random,
            &self.shadow,
            pickup_rate,
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

pub fn configure_mouse_signal(
    canvas_ref: NodeRef<Canvas>,
    set_mouse: WriteSignal<(bool, f32, f32)>,
    canvas_w: u32,
    canvas_h: u32,
) {
    let evt_options = UseEventListenerOptions::default().passive(true);
    let to_tex = move |x: i32, y: i32| (x as f32 / canvas_w as f32, 1.0 - y as f32 / canvas_h as f32);
    on_cleanup(use_event_listener_with_options(
        canvas_ref,
        leptos::ev::mousedown,
        move |evt| {
            let coords = to_tex(evt.offset_x(), evt.offset_y());
            *set_mouse.write() = (true, coords.0, coords.1);
        },
        evt_options,
    ));
    on_cleanup(use_event_listener_with_options(
        window(),
        leptos::ev::mouseup,
        move |_| {
            set_mouse.update(|tup| tup.0 = false);
        },
        evt_options,
    ));
    on_cleanup(use_event_listener_with_options(
        canvas_ref,
        leptos::ev::mousemove,
        move |evt| {
            set_mouse.update(|tup| {
                if tup.0 {
                    let coords = to_tex(evt.offset_x(), evt.offset_y());
                    tup.1 = coords.0;
                    tup.2 = coords.1;
                }
            });
        },
        evt_options,
    ));
    on_cleanup(use_event_listener_with_options(
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
            let coords = to_tex(
                touch.client_x() - rect.x() as i32,
                touch.client_y() - rect.y() as i32,
            );
            *set_mouse.write() = (true, coords.0, coords.1);
        },
        evt_options,
    ));
    on_cleanup(use_event_listener_with_options(
        canvas_ref,
        leptos::ev::touchend,
        move |_| set_mouse.update(|tup| tup.0 = false),
        evt_options,
    ));
    on_cleanup(use_event_listener_with_options(
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
                if tup.0 {
                    let coords = to_tex(
                        touch.client_x() - rect.x() as i32,
                        touch.client_y() - rect.y() as i32,
                    );
                    tup.1 = coords.0;
                    tup.2 = coords.1;
                }
            });
        },
        evt_options,
    ));
}

#[component]
pub fn App() -> impl IntoView {
    let canvas_w: u32 = 736;
    let canvas_h: u32 = 736;

    let dial_w: u32 = canvas_w / 2;
    let dial_h: u32 = canvas_h / 2;
    let frame_period = 8.333;

    let main_canvas_ref = NodeRef::<Canvas>::new();
    let wind_dial_canvas_ref = NodeRef::<Canvas>::new();
    let sun_dial_canvas_ref = NodeRef::<Canvas>::new();

    let (main_cursor, set_main_cursor) = signal((false, 0.0, 0.0));
    let (wind_dial_cursor, set_wind_dial_cursor) = signal((false, 0.5, 0.5));
    let (sun_dial_cursor, set_sun_dial_cursor) = signal((false, 0.75, 0.75));
    let (next_frame, set_next_frame) = signal(());

    configure_mouse_signal(main_canvas_ref, set_main_cursor, canvas_w, canvas_h);
    configure_mouse_signal(wind_dial_canvas_ref, set_wind_dial_cursor, dial_w, dial_h);
    configure_mouse_signal(sun_dial_canvas_ref, set_sun_dial_cursor, dial_w, dial_h);

    request_animation_frame(move || {
        *set_next_frame.write();
    });

    Effect::new(move || {
        next_frame.get();
        request_animation_frame(move || {
            if !set_next_frame.is_disposed() {
                *set_next_frame.write();
            }
        });
    });

    let next_frame_throttled: Signal<()> = signal_throttled(next_frame, frame_period);
    let wind_direction = Signal::derive(move || {
        let tup = wind_dial_cursor.get();
        let mut x = -(tup.1 - 0.5) * 2.0;
        let mut y = -(tup.2 - 0.5) * 2.0;
        if x == 0.0 {
            x += 0.01;
        }
        if y == 0.0 {
            y+= 0.01;
        }
        let mut magnitude = x.hypot(y);
        let tangent =  38.0f32.to_radians().tan();
        if magnitude > 1.0 {
            x /= magnitude;
            y /= magnitude;
            magnitude = 1.0;
        }
        (x, y, tangent * magnitude)
    });
    let sun_position_polar = Signal::derive(move || {
        let tup = sun_dial_cursor.get();
        let mut x = (tup.1 - 0.5) * 2.0;
        let mut y = (tup.2 - 0.5) * 2.0;
        if x == 0.0 {
            x += 0.01;
        }
        if y == 0.0 {
            y += 0.01;
        }
        (
            1.0,
            y.atan2(x),
            (1.0 - (x.powi(2) + y.powi(2)).min(1.0)).sqrt().acos(),
        )
    });
    let sand_drop_location = Signal::derive(move || {
        let tup = main_cursor.get();
        if tup.0 {
            return Some((tup.1, tup.2));
        } else {
            return None;
        }
    });

    Effect::new(move |_| {
        if let Some(canvas) = main_canvas_ref.get() {
            canvas.set_width(canvas_w);
            canvas.set_height(canvas_h);
            let context = canvas
                .get_context("webgl2")
                .expect("get_context")
                .expect("object")
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            canvas_fill(
                context.clone(),
                next_frame_throttled.into(),
                sand_drop_location.into(),
                sun_position_polar.into(),
                wind_direction.into(),
            );
        }
    });

    Effect::new(move |_| {
        if let Some(canvas) = sun_dial_canvas_ref.get() {
            canvas.set_width(dial_w);
            canvas.set_height(dial_h);
            let context = canvas
                .get_context("webgl2")
                .expect("get_context")
                .expect("object")
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            sun_dial_fill(
                context.clone(),
                next_frame_throttled.into(),
                sun_dial_cursor.into(),
            );
        }
    });

    Effect::new(move |_| {
        if let Some(canvas) = wind_dial_canvas_ref.get() {
            canvas.set_width(dial_w);
            canvas.set_height(dial_h);
            let context = canvas
                .get_context("webgl2")
                .expect("get_context")
                .expect("object")
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            wind_dial_fill(
                context.clone(),
                next_frame_throttled.into(),
                wind_dial_cursor.into(),
            );
        }
    });

    view! {
        <h1 style:margin="40px">"WebGL Dune Saltation"</h1>
        <canvas style:touch-action="pinch-zoom" node_ref=main_canvas_ref />
        <canvas style:touch-action="pinch-zoom" node_ref=sun_dial_canvas_ref />
        <canvas style:touch-action="pinch-zoom" node_ref=wind_dial_canvas_ref />
        <h2 style:margin="40px">"Written by Jackson Welles"</h2>
        <h2 style:margin="40px">"Base saltation algorithm from Brad Werner, via "
            <a href="https://smallpond.ca/jim/sand/dunefieldMorphology/index.html"> "Jim Elder's excellent write up." </a> </h2>

    }
}

fn wind_dial_fill(
    context: WebGl2RenderingContext,
    next_frame: Signal<()>,
    cursor: Signal<(bool, f32, f32)>,
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

    let quad = Rc::new(Quad::create(&context));

    let wind_dial_pipeline =
        WindDialPipeline::create(&context, make_prog(include_str!("shaders/wind_dial.frag")));

    let mut wind_dial_stage = WindDialStage {
        context: context.clone(),
        quad: quad,
        pipeline: wind_dial_pipeline,
    };

    Effect::new(move || {
        next_frame.get();
        let (_, x, y) = cursor.get_untracked();
        wind_dial_stage.update((x, y));
    });
}

fn sun_dial_fill(
    context: WebGl2RenderingContext,
    next_frame: Signal<()>,
    cursor: Signal<(bool, f32, f32)>,
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

    let quad = Rc::new(Quad::create(&context));

    let sun_dial_pipeline =
        SunDialPipeline::create(&context, make_prog(include_str!("shaders/sun_dial.frag")));

    let mut sun_dial_stage = SunDialStage {
        context: context.clone(),
        quad: quad,
        pipeline: sun_dial_pipeline,
    };

    Effect::new(move || {
        next_frame.get();
        let (_, x, y) = cursor.get_untracked();
        sun_dial_stage.update((x, y));
    });
}

fn canvas_fill(
    context: WebGl2RenderingContext,
    next_frame: Signal<()>,
    sand_drop_location: Signal<Option<(f32, f32)>>,
    sun_position_polar: Signal<(f32, f32, f32)>,
    wind_direction: Signal<(f32, f32, f32)>,
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
    let radius = 200.0;

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

    let shadow = Rc::new(make_shadow(&context, window_w, window_h));
    let random = Rc::new(make_rand(&context, sand_w, sand_h));
    let delta_sand = make_sand(&context, sand_w, sand_h);
    let sand = Rc::new(RefCell::new(make_sand(&context, sand_w, sand_h)));
    let lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, window_w, window_h,
    )));

    // let wind_shadow = Rc::new(make_shadow(&context, sand_w / 2, sand_h / 2));
    // let wind_shadow_lookahead = Rc::new(RefCell::new(make_shadow_lookahead(&context, sand_w / 2, sand_h / 2)));

    let wind_shadow = Rc::new(make_shadow(&context, sand_w, sand_h));
    let wind_shadow_lookahead = Rc::new(RefCell::new(make_shadow_lookahead(
        &context, sand_w, sand_h,
    )));

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
    };

    Effect::new(move || {
        next_frame.get();
        let (r, theta, phi) = sun_position_polar.get_untracked();
        let r_sin_phi = r.mul(phi.sin());
        let sun_x = r_sin_phi.mul(theta.cos());
        let sun_y = r_sin_phi.mul(theta.sin());
        let sun_z = r.mul(phi.cos());
        if let Some((sand_x, sand_y)) = sand_drop_location.get_untracked() {
            drop_stage.update(sand_x, sand_y);
        }

        let wind_dir = wind_direction.get_untracked();
        let magnitude = wind_dir.0.hypot(wind_dir.1);

        avalanche_stage.update();
        lookahead_stage.update((sun_x, sun_y));
        shadow_stage.update((sun_x, sun_y, sun_z));

        // if wind_dir.0.is_normal() && wind_dir.1.is_normal() {
            wind_lookahead_stage.update((wind_dir.0, wind_dir.1));
            wind_shadow_stage.update(wind_dir);

            wind_stage.update((wind_dir.0, wind_dir.1), 0.0015 * magnitude,  if magnitude > 0.2 {(magnitude - 0.2).mul(1.2).min(0.6)} else {0.0});
        // }
        draw_stage.update((sun_x, sun_y, sun_z));
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
