use std::collections::HashMap;

use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlUniformLocation,
};
use web_sys::{WebGlFramebuffer, WebGlTexture, js_sys};

type GL = WebGl2RenderingContext;
pub struct TexelSize {
    pub x: f32,
    pub y: f32,
}

pub trait JsView {
    unsafe fn to_js_obj(&self) -> js_sys::Object;
}

pub trait JsViewMut: JsView {
    unsafe fn to_mut_js_obj(&mut self) -> js_sys::Object;
}

pub struct ArrayView<'a, T> {
    data: &'a [T],
}

pub struct ArrayViewMut<'a, T> {
    data: &'a mut [T],
}

impl<'a, T> ArrayView<'a, T> {
    pub fn create(data: &'a [T]) -> ArrayView<'a, T> {
        ArrayView { data }
    }
}

impl<'a, T> ArrayViewMut<'a, T> {
    pub fn create(data: &'a mut [T]) -> ArrayViewMut<'a, T> {
        ArrayViewMut { data }
    }
}

impl JsView for ArrayView<'_, f32> {
    unsafe fn to_js_obj(&self) -> js_sys::Object {
        unsafe {
            // wasm memory is one big block that can get resized
            // whenever an alloc happens, this view takes a raw
            // pointer into our wasm memory, so we must be careful
            // not to alloc between taking this pointer and giving
            // it to the js api
            js_sys::Float32Array::view(self.data).into()
        }
    }
}

impl JsView for ArrayView<'_, u8> {
    unsafe fn to_js_obj(&self) -> js_sys::Object {
        unsafe {
            // wasm memory is one big block that can get resized
            // whenever an alloc happens, this view takes a raw
            // pointer into our wasm memory, so we must be careful
            // not to alloc between taking this pointer and giving
            // it to the js api
            js_sys::Uint8Array::view(self.data).into()
        }
    }
}

impl JsViewMut for ArrayViewMut<'_, f32> {
    unsafe fn to_mut_js_obj(&mut self) -> js_sys::Object {
        unsafe {
            js_sys::Float32Array::view_mut_raw(self.data.as_mut_ptr(), self.data.len()).into()
        }
    }
}

impl JsView for ArrayViewMut<'_, f32> {
    unsafe fn to_js_obj(&self) -> js_sys::Object {
        unsafe { js_sys::Float32Array::view(self.data).into() }
    }
}

pub struct BufferedTexture {
    context: WebGl2RenderingContext,
    texture: Option<WebGlTexture>,
    framebuffer: Option<WebGlFramebuffer>,
    internal_format: u32,
    width: i32,
    height: i32,
    texel_size: TexelSize,
}

impl BufferedTexture {
    pub fn create<T: JsView>(
        context: &WebGl2RenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        src_data: Option<T>,
        tex_params: &[(u32, u32)],
    ) -> Self {
        let texture = context.create_texture();
        context.bind_texture(GL::TEXTURE_2D, texture.as_ref());

        unsafe {
            context
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                target,
                level,
                internal_format as i32,
                width,
                height,
                border,
                format,
                data_type,
                src_data.map(|view|view.to_js_obj()).as_ref(),
            )
            .expect("failed to create texture");
        }

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
        BufferedTexture {
            context: context.clone(),
            texture,
            framebuffer,
            internal_format,
            width,
            height,
            texel_size: TexelSize {
                x: 1.0 / width as f32,
                y: 1.0 / height as f32,
            },
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn texel_size(&self) -> &TexelSize {
        &self.texel_size
    }

    pub fn attach(&self, id: i32) -> i32 {
        self.context.active_texture(GL::TEXTURE0 + id as u32);
        self.context
            .bind_texture(GL::TEXTURE_2D, self.texture.as_ref());
        id
    }

    pub fn copy_from(&self, other: &BufferedTexture) -> Result<(), &str> {
        if self.width != other.width || self.height != other.height {
            return Err("mismatched sizes");
        }
        if self.internal_format != other.internal_format {
            return Err("mismatched formats");
        }
        self.context
            .bind_framebuffer(GL::FRAMEBUFFER, other.framebuffer.as_ref());
        self.context
            .bind_texture(GL::TEXTURE_2D, self.texture.as_ref());
        self.context.copy_tex_image_2d(
            GL::TEXTURE_2D,
            0,
            self.internal_format,
            0,
            0,
            self.width,
            self.height,
            0,
        );
        Ok(())
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
    pub fn create<T: JsView>(
        context: &WebGl2RenderingContext,
        target: u32,
        level: i32,
        internal_format: u32,
        width: i32,
        height: i32,
        border: i32,
        format: u32,
        data_type: u32,
        src_data: Option<T>,
        tex_params: &[(u32, u32)],
    ) -> Self {
        SwappableTexture {
            first: BufferedTexture::create(
                context,
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
                context,
                target,
                level,
                internal_format,
                width,
                height,
                border,
                format,
                data_type,
                None::<T>,
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

pub struct Quad {
    context: WebGl2RenderingContext,
    buff: Option<WebGlBuffer>,
}

impl Quad {
    pub fn create(context: &WebGl2RenderingContext) -> Self {
        let quad_vertices: [f32; 12] = [
            -1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0,
        ];
        let buff = context.create_buffer();
        context.bind_buffer(GL::ARRAY_BUFFER, buff.as_ref());
        unsafe {
            let quad_vert_view = js_sys::Float32Array::view(&quad_vertices);
            context.buffer_data_with_array_buffer_view(
                GL::ARRAY_BUFFER,
                &quad_vert_view,
                GL::STATIC_DRAW,
            );
        }
        Quad {
            context: context.clone(),
            buff,
        }
    }

    pub fn blit(&self, target: Option<&BufferedTexture>) {
        self.context
            .bind_buffer(GL::ARRAY_BUFFER, self.buff.as_ref());

        self.context
            .vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
        self.context.enable_vertex_attrib_array(0);

        match target {
            Some(tex) => {
                self.context.viewport(0, 0, tex.width, tex.height);
                self.context
                    .bind_framebuffer(GL::FRAMEBUFFER, tex.framebuffer.as_ref());
            }
            None => {
                self.context.viewport(
                    0,
                    0,
                    self.context.drawing_buffer_width(),
                    self.context.drawing_buffer_height(),
                );
                self.context.bind_framebuffer(GL::FRAMEBUFFER, None);
            }
        }

        self.context.clear_color(0.0, 0.0, 0.0, 1.0);
        self.context.clear(GL::COLOR_BUFFER_BIT);

        self.context.draw_arrays(GL::TRIANGLES, 0, 6);
    }
}

impl Drop for Quad {
    fn drop(&mut self) {
        self.context.delete_buffer(self.buff.as_ref());
    }
}
