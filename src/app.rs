use leptos::html::Canvas;
use leptos::prelude::*;
use leptos::wasm_bindgen::prelude::*;

use web_sys::js_sys;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

type GL = WebGl2RenderingContext;

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref = NodeRef::<Canvas>::new();
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            canvas.set_width(480);
            canvas.set_height(480);
            let context = canvas
                .get_context("webgl2")
                .expect("get_context")
                .expect("object")
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            canvas_fill(context);
        }
    });

    view! { <canvas node_ref=canvas_ref />  }
}

fn canvas_fill(context: WebGl2RenderingContext) {
    let vert_shader = compile_shader(
        &context,
        GL::VERTEX_SHADER,
        r##"
        attribute vec4 a_position;
        attribute vec2 a_texcoord;

        varying vec2 u_texcoord;

        void main() {
            gl_Position = a_position;
            u_texcoord = a_texcoord;
        }
        "##,
    )
    .unwrap();

    let frag_shader = compile_shader(
        &context,
        GL::FRAGMENT_SHADER,
        r##"
        precision mediump float;
        
        varying vec2 u_texcoord;
        uniform sampler2D u_texture;
        
        void main() {
            gl_FragColor = texture2D(u_texture, u_texcoord);
        }
        "##,
    )
    .unwrap();
    let program = link_program(&context, &vert_shader, &frag_shader).unwrap();

    let position_attribute_location = context.get_attrib_location(&program, "a_position") as u32;
    let texcoord_attribute_location = context.get_attrib_location(&program, "a_texcoord") as u32;
    let texture_uniform_location = context.get_uniform_location(&program, "u_texture");

    let position_buffer = context.create_buffer();
    context.bind_buffer(GL::ARRAY_BUFFER, position_buffer.as_ref());
    set_geometry(&context);

    let texcoord_buffer = context.create_buffer();
    context.bind_buffer(GL::ARRAY_BUFFER, texcoord_buffer.as_ref());
    set_texcoords(&context);

    let base_texture = context.create_texture();
    context.bind_texture(GL::TEXTURE_2D, base_texture.as_ref());
    set_base_texture(&context);

    let vao = context.create_vertex_array();
    context.bind_vertex_array(vao.as_ref());

    context.use_program(Some(&program));

    context.enable_vertex_attrib_array(position_attribute_location);
    context.bind_buffer(GL::ARRAY_BUFFER, position_buffer.as_ref());

    context.vertex_attrib_pointer_with_i32(position_attribute_location, 3, GL::FLOAT, false, 0, 0);

    context.enable_vertex_attrib_array(texcoord_attribute_location);
    context.bind_buffer(GL::ARRAY_BUFFER, texcoord_buffer.as_ref());

    context.vertex_attrib_pointer_with_i32(texcoord_attribute_location, 2, GL::FLOAT, false, 0, 0);

    context.bind_texture(GL::TEXTURE_2D, base_texture.as_ref());
    context.uniform1i(texture_uniform_location.as_ref(), 0);

    draw(&context, 6);
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

fn set_base_texture(context: &WebGl2RenderingContext) {
    let texture_data: [u8; 6] = [128, 64, 128, 0, 192, 0];

    context.pixel_storei(GL::UNPACK_ALIGNMENT, 1);

    context
        .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
            GL::TEXTURE_2D,
            0,
            GL::LUMINANCE as i32,
            3,
            2,
            0,
            GL::LUMINANCE,
            GL::UNSIGNED_BYTE,
            &texture_data,
            0,
        )
        .expect("failed to create texture");

    context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
    context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
    context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
    context.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
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
