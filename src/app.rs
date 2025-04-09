use std::cell::RefCell;
use std::rc::Rc;

use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
use web_sys::{WebGlFramebuffer, WebGlTexture, js_sys};

type GL = WebGl2RenderingContext;

struct SwappableTexture {
    context: WebGl2RenderingContext,
    a_texture: Option<WebGlTexture>,
    b_texture: Option<WebGlTexture>,
    a_framebuff: Option<WebGlFramebuffer>,
    b_framebuff: Option<WebGlFramebuffer>,
    from_a: bool,
}

impl SwappableTexture {
    fn create(
        context: WebGl2RenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        src_data: Option<&[u8]>,
        tex_params: &[(u32, u32)],
    ) -> Self {
        let make_texture = |use_data: bool| {
            let texture = context.create_texture();
            context.bind_texture(GL::TEXTURE_2D, texture.as_ref());
            if use_data {
                context.pixel_storei(GL::UNPACK_ALIGNMENT, 1);
            }
            context
                .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                    target,
                    level,
                    internal_format as i32,
                    width,
                    height,
                    border,
                    format,
                    data_type,
                    if use_data { src_data } else { None },
                )
                .expect("failed to create texture");

            for (key, value) in tex_params {
                context.tex_parameteri(GL::TEXTURE_2D, *key, *value as i32);
            }
            texture
        };
        let make_framebuffer = |texture: &Option<WebGlTexture>| {
            let framebuffer = context.create_framebuffer();
            context.bind_framebuffer(GL::FRAMEBUFFER, framebuffer.as_ref());
            context.framebuffer_texture_2d(
                GL::FRAMEBUFFER,
                GL::COLOR_ATTACHMENT0,
                GL::TEXTURE_2D,
                texture.as_ref(),
                0,
            );
            framebuffer
        };
        let a_texture = make_texture(true);
        let b_texture = make_texture(false);

        let a_framebuff = make_framebuffer(&a_texture);
        let b_framebuff = make_framebuffer(&b_texture);

        SwappableTexture {
            context,
            a_texture,
            b_texture,
            a_framebuff,
            b_framebuff,
            from_a: true,
        }
    }

    fn bind_tex(&self) {
        self.context.bind_texture(
            GL::TEXTURE_2D,
            if self.from_a {
                self.a_texture.as_ref()
            } else {
                self.b_texture.as_ref()
            },
        );
    }

    fn bind_fb(&self) {
        self.context.bind_framebuffer(
            GL::FRAMEBUFFER,
            if self.from_a {
                self.b_framebuff.as_ref()
            } else {
                self.a_framebuff.as_ref()
            },
        );
    }

    fn bind_all(&self) {
        self.bind_tex();
        self.bind_fb();
    }

    fn swap(&mut self) {
        self.from_a = !self.from_a;
    }
}

impl Drop for SwappableTexture {
    fn drop(&mut self) {
        self.context.delete_framebuffer(self.b_framebuff.as_ref());
        self.context.delete_framebuffer(self.a_framebuff.as_ref());
        self.context.delete_texture(self.a_texture.as_ref());
        self.context.delete_texture(self.b_texture.as_ref());
    }
}

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
        attribute vec4 a_position;
        attribute vec2 a_texcoord;

        varying vec2 v_texcoord;

        void main() {
            gl_Position = a_position;
            v_texcoord = a_texcoord;
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
    let quad_program = link_program(&context, &quad_vert_shader, &quad_frag_shader).unwrap();
    let life_program = link_program(&context, &quad_vert_shader, &life_frag_shader).unwrap();

    let quad_texture_location = context.get_uniform_location(&quad_program, "u_texture");
    let quad_position_location = context.get_attrib_location(&quad_program, "a_position") as u32;
    let quad_texcoord_location = context.get_attrib_location(&quad_program, "a_texcoord") as u32;

    let life_texture_location = context.get_uniform_location(&life_program, "u_texture");
    let life_texel_size_location = context.get_uniform_location(&life_program, "u_texel_size");
    let life_position_location = context.get_attrib_location(&life_program, "a_position") as u32;
    let life_texcoord_location = context.get_attrib_location(&life_program, "a_texcoord") as u32;

    let position_buffer = context.create_buffer();
    context.bind_buffer(GL::ARRAY_BUFFER, position_buffer.as_ref());
    set_geometry(&context);

    let texcoord_buffer = context.create_buffer();
    context.bind_buffer(GL::ARRAY_BUFFER, texcoord_buffer.as_ref());
    set_texcoords(&context);

    let mut swap_texture = make_swap_texture(context.clone());

    let vao = context.create_vertex_array();
    context.bind_vertex_array(vao.as_ref());

    context.bind_buffer(GL::ARRAY_BUFFER, position_buffer.as_ref());
    context.enable_vertex_attrib_array(quad_position_location);
    context.vertex_attrib_pointer_with_i32(quad_position_location, 3, GL::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(life_position_location);
    context.vertex_attrib_pointer_with_i32(life_position_location, 3, GL::FLOAT, false, 0, 0);

    context.bind_buffer(GL::ARRAY_BUFFER, texcoord_buffer.as_ref());
    context.enable_vertex_attrib_array(quad_texcoord_location);
    context.vertex_attrib_pointer_with_i32(quad_texcoord_location, 2, GL::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(life_texcoord_location);
    context.vertex_attrib_pointer_with_i32(life_texcoord_location, 2, GL::FLOAT, false, 0, 0);

    context.use_program(Some(&life_program));

    context.uniform1i(life_texture_location.as_ref(), 0);
    context.uniform1f(life_texel_size_location.as_ref(), 1.0 / 32.0);

    context.use_program(Some(&quad_program));

    context.uniform1i(quad_texture_location.as_ref(), 0);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut prev_time = None::<f64>;

    *g.borrow_mut() = Some(Closure::new(move || {
        let now = window().performance().unwrap().now();
        if !prev_time.is_some() || now - prev_time.unwrap() > 50.0 {
            prev_time = Some(now);

            context.use_program(Some(&quad_program));
            swap_texture.bind_tex();
            context.bind_framebuffer(GL::FRAMEBUFFER, None);
            context.viewport(0, 0, 512, 512);
            draw(&context, 6);

            context.use_program(Some(&life_program));
            swap_texture.bind_fb();
            context.viewport(0, 0, 32, 32);
            draw(&context, 6);

            swap_texture.swap();

            context.use_program(Some(&quad_program));
            swap_texture.bind_all();
            context.viewport(0, 0, 32, 32);

            draw(&context, 6);

            swap_texture.swap();
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

fn set_geometry(context: &WebGl2RenderingContext) {
    let quad_vertices: [f32; 18] = [
        -1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 1.0, -1.0,
        0.0,
    ];

    // we cannot allocate any new memory between view
    unsafe {
        let quad_verts_view = js_sys::Float32Array::view(&quad_vertices);

        context.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &quad_verts_view,
            GL::STATIC_DRAW,
        );
    }
}

fn set_texcoords(context: &WebGl2RenderingContext) {
    let quad_texture_coords: [f32; 12] =
        [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0];

    unsafe {
        let quad_coords_view = js_sys::Float32Array::view(&quad_texture_coords);

        context.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &quad_coords_view,
            GL::STATIC_DRAW,
        );
    }
}

fn make_swap_texture(context: WebGl2RenderingContext) -> SwappableTexture {
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

    return SwappableTexture::create(
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

fn draw(context: &WebGl2RenderingContext, vert_count: i32) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(GL::COLOR_BUFFER_BIT);

    context.draw_arrays(GL::TRIANGLES, 0, vert_count);
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
