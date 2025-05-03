use leptos::prelude::*;

use std::collections::HashMap;

use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};
use web_sys::{WebGlFramebuffer, WebGlTexture, js_sys};

type GL = WebGl2RenderingContext;
pub struct TexelSize {
    pub x: f32,
    pub y: f32,
}
pub struct BufferedTexture {
    context: WebGl2RenderingContext,
    texture: Option<WebGlTexture>,
    framebuffer: Option<WebGlFramebuffer>,
    width: i32,
    height: i32,
    texel_size: TexelSize,
}

impl BufferedTexture {
    pub fn create(
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
        let texture = context.create_texture();
        context.bind_texture(GL::TEXTURE_2D, texture.as_ref());
        if src_data.is_some() {
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
                src_data,
            )
            .expect("failed to create texture");

        for (key, value) in tex_params {
            context.tex_parameteri(GL::TEXTURE_2D, *key, *value as i32);
        }
        let framebuffer = context.create_framebuffer();
        context.bind_framebuffer(GL::FRAMEBUFFER, framebuffer.as_ref());
        context.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::COLOR_ATTACHMENT0,
            GL::TEXTURE_2D,
            texture.as_ref(),
            0,
        );
        return BufferedTexture {
            context,
            texture,
            framebuffer,
            width,
            height,
            texel_size: TexelSize {
                x: 1.0 / width as f32,
                y: 1.0 / height as f32,
            },
        };
    }

    pub fn texel_size(&self) -> & TexelSize {
        &self.texel_size
    }

    pub fn attach(&self, id: i32) -> i32 {
        self.context.active_texture(GL::TEXTURE0 + id as u32);
        self.context
            .bind_texture(GL::TEXTURE_2D, self.texture.as_ref());
        id
    }
}

impl Drop for BufferedTexture {
    fn drop(&mut self) {
        self.context.delete_framebuffer(self.framebuffer.as_ref());
        self.context.delete_texture(self.texture.as_ref());
    }
}

pub struct SwappableTexture {
    first: BufferedTexture,
    second: BufferedTexture,
    parity: bool,
}

impl SwappableTexture {
    const START: bool = true;
    pub fn create(
        context: &WebGl2RenderingContext,
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
        SwappableTexture {
            first: BufferedTexture::create(
                context.clone(),
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                data_type,
                src_data,
                tex_params,
            ),
            second: BufferedTexture::create(
                context.clone(),
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                data_type,
                None,
                tex_params,
            ),
            parity: Self::START,
        }
    }

    pub fn read(&self) -> &BufferedTexture {
        if self.parity == Self::START {
            &self.first
        } else {
            &self.second
        }
    }

    pub fn write(&self) -> &BufferedTexture {
        if self.parity == Self::START {
            &self.second
        } else {
            &self.first
        }
    }

    pub fn texel_size(&self) -> &TexelSize {
        self.first.texel_size()
    }

    pub fn swap(&mut self) {
        self.parity = !self.parity;
    }
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

pub struct Program {
    program: WebGlProgram,
    uniforms: HashMap<String, WebGlUniformLocation>,
}

impl Program {
    pub fn create(
        context: &WebGl2RenderingContext,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Self {
        let program = context
            .create_program()
            .expect("Unable to create program object");

        context.attach_shader(&program, vert_shader);
        context.attach_shader(&program, frag_shader);
        context.link_program(&program);

        context
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .expect(
                &context
                    .get_program_info_log(&program)
                    .unwrap_or(String::from("Unknown error linking program")),
            );

        let uniform_count = context
            .get_program_parameter(&program, GL::ACTIVE_UNIFORMS)
            .as_f64()
            .unwrap() as u32;

        let mut uniforms = HashMap::new();
        for i in 0..uniform_count {
            let name = context.get_active_uniform(&program, i).unwrap().name();
            let location = context.get_uniform_location(&program, &name).unwrap();
            uniforms.insert(name, location);
        }

        Program { program, uniforms }
    }

    pub fn uniforms(&self) -> &HashMap<String, WebGlUniformLocation> {
        &self.uniforms
    }

    pub fn program(&self) -> &WebGlProgram {
        &self.program
    }
}

pub fn blit(context: &WebGl2RenderingContext, target: Option<&BufferedTexture>) {
    let quad_vertices: [f32; 12] = [
        -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0,
    ];
    context.bind_buffer(GL::ARRAY_BUFFER, context.create_buffer().as_ref());

    unsafe {
        let quad_vert_view = js_sys::Float32Array::view(&quad_vertices);
        context.buffer_data_with_array_buffer_view(
            GL::ARRAY_BUFFER,
            &quad_vert_view,
            GL::STATIC_DRAW,
        );
    }

    context.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(0);

    match target {
        Some(tex) => {
            context.viewport(0, 0, tex.width, tex.height);
            context.bind_framebuffer(GL::FRAMEBUFFER, tex.framebuffer.as_ref());
        }
        None => {
            context.viewport(
                0,
                0,
                context.drawing_buffer_width(),
                context.drawing_buffer_height(),
            );
            context.bind_framebuffer(GL::FRAMEBUFFER, None);
        }
    }

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(GL::COLOR_BUFFER_BIT);

    context.draw_arrays(GL::TRIANGLES, 0, 6);
}
