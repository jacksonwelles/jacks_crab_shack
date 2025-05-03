use super::common::*;

use std::cell::RefCell;
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
    let quad_vert_shader = compile_shader(
        &context,
        GL::VERTEX_SHADER,
        r##"
        attribute vec2 a_position;
        varying vec2 v_texcoord;

        void main() {
            gl_Position = vec4(a_position, 0, 0);
            v_texcoord = a_position * 0.5 + 0.5;
        }
        "##,
    )
    .unwrap();

    let quad_frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        r##"
        precision mediump float;

        varying vec2 v_texcoord;
        uniform sampler2D u_texture;

        void main() {
            gl_FragColor = texture2D(u_texture, v_texcoord);
        }
        "##,
    )
    .unwrap();

    let life_frag_shader = compile_shader(&context, GL::FRAGMENT_SHADER,
    r##"
        precision mediump float;

        varying vec2 v_texcoord;
        uniform sampler2D u_texture;
        uniform float u_texel_size;

        void main() {
            int sum = 0;
            bool alive = texture2D(u_texture, v_texcoord).r > 0.0;
            gl_FragColor = vec4(0,0,0,1);
            sum += int(texture2D(u_texture, v_texcoord + vec2( 0            , u_texel_size  )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size , u_texel_size  )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size , 0             )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size , -u_texel_size )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( 0            , -u_texel_size )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size, -u_texel_size )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size, 0             )).r > 0.0);
            sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size, u_texel_size  )).r > 0.0);
            if (alive) {
                if (sum == 2 || sum == 3) {
                    gl_FragColor = vec4(1,1,1,0);
                }
            } else if (sum == 3) {
                gl_FragColor = vec4(1,1,1,0);
            }
            // if (alive) {
            //     gl_FragColor = vec4(1,1,1,1);
            // }
            // gl_FragColor = texture2D(u_texture, v_texcoord);
        }
    "##).unwrap();
    let quad_program = Program::create(&context, &quad_vert_shader, &quad_frag_shader);
    let life_program = Program::create(&context, &quad_vert_shader, &life_frag_shader);

    let mut game_board = make_game_board(context.clone());

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut prev_time = None::<f64>;

    *g.borrow_mut() = Some(Closure::new(move || {
        let now = window().performance().unwrap().now();
        if !prev_time.is_some() || now - prev_time.unwrap() > 5000.0 {
            prev_time = Some(now);

            context.use_program(Some(quad_program.program()));
            context.uniform1i(quad_program.uniforms().get("u_texture"), game_board.attach(0));
            blit(&context, None);

            // context.use_program(Some(life_program.program()));
            // context.uniform1i(life_program.uniforms().get("u_texture"), game_board.read().attach(0));
            // context.uniform2f(life_program.uniforms().get("u_texel_size"), game_board.texel_size().x, game_board.texel_size().y);
            // blit(&context, Some(&game_board.write()));

            // game_board.swap();
        }

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



fn make_game_board(context: WebGl2RenderingContext) -> BufferedTexture {
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

    return BufferedTexture::create(
        context,
        GL::TEXTURE_2D,
        0,
        GL::RGBA,
        32,
        32,
        0,
        GL::RGBA,
        GL::UNSIGNED_BYTE,
        Some(&texture_data),
        &[
            (GL::TEXTURE_MIN_FILTER, GL::NEAREST),
            (GL::TEXTURE_MAG_FILTER, GL::NEAREST),
            (GL::TEXTURE_WRAP_S, GL::REPEAT),
            (GL::TEXTURE_WRAP_T, GL::REPEAT),
        ],
    );
}

