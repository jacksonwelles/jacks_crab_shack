use proc_macro::{TokenStream, TokenTree, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::Ident;

use std::fs;
use std::path::Path;

#[derive(PartialEq)]
enum UniformType {
    Sampler2D,
    Float,
    Vec2,
    Vec3,
    Vec4,
}

struct Uniform {
    name: proc_macro2::TokenStream,
    u_type: UniformType,
}

fn get_type_token(u_type: &UniformType) -> proc_macro2::TokenStream {
    match u_type {
        UniformType::Sampler2D => "&BufferedTexture",
        UniformType::Float => "f32",
        UniformType::Vec2 => "(f32, f32)",
        UniformType::Vec3 => "(f32, f32, f32)",
        UniformType::Vec4 => "(f32, f32, f32, f32)",
    }
    .parse()
    .unwrap()
}

fn get_uniform_type(type_name: &str) -> Result<UniformType, String> {
    match type_name {
        "sampler2D" => Ok(UniformType::Sampler2D),
        "float" => Ok(UniformType::Float),
        "vec2" => Ok(UniformType::Vec2),
        "vec3" => Ok(UniformType::Vec3),
        "vec4" => Ok(UniformType::Vec4),
        _ => Err(format!("could not parse type: '{}'", type_name)),
    }
}

fn make_setup_step(idx: i32, uniform: &Uniform) -> Option<proc_macro2::TokenStream> {
    let name = &uniform.name;
    match uniform.u_type {
        UniformType::Sampler2D => Some(quote! {
            context.uniform1i(
                program
                    .uniforms()
                    .get(stringify!(#name))
                    .unwrap()
                    .into(),
                #idx
            )
        }),
        _ => None,
    }
}

fn make_update_step(idx: i32, uniform: &Uniform) -> proc_macro2::TokenStream {
    let name = &uniform.name;
    if uniform.u_type == UniformType::Sampler2D {
        return quote! {
            #name.attach(#idx)
        }
    }
    let uniform_location = quote! {
        self.program.uniforms().get(stringify!(#name)).unwrap().into()
    };
    let uniform_update = match uniform.u_type {
        UniformType::Float => quote!{
            context.uniform1f(#uniform_location, #name)
        },
        UniformType::Vec2 => quote!{
            context.uniform2f(#uniform_location, #name.0, #name.1)
        },
        UniformType::Vec3 => quote!{
            context.uniform3f(#uniform_location, #name.0, #name.1, #name.2)
        },
        UniformType::Vec4 => quote!{
            context.uniform4f(#uniform_location, #name.0, #name.1, #name.2, #name.3)
        },
        _=> unreachable!()
    };

    quote! {
        if self.#name != #name {
            #uniform_update;
            self.#name = #name;
        }
    }
}

fn parse_shader_path(path: &str, span: &proc_macro::Span) -> Result<Vec<Uniform>, String> {

    if !path.starts_with('"') || !path.ends_with('"') {
        return Err("shader path must be a string literal".to_string());
    }
    let unquoted_path = &path[1..(path.len() - 1)];

    let mut call_file = span.file();
    if call_file.is_empty() {
        // probably from the analyzer, try something and hope it sticks...
        call_file = "src/foo".to_string();
    }

    let full_path = Path::new(&call_file).with_file_name(unquoted_path);
    let source = fs::read_to_string(full_path).map_err(|e| e.to_string())?;
    let mut split = source.split(&[' ', '\r', '\n', '\t']);
    let mut result = Vec::new();
    while let Some(str) = split.next() {
        if str != "uniform" {
            continue;
        }
        let u_type = get_uniform_type(split.next().ok_or("no uniform type")?)?;
        let mut name = split.next().ok_or("unamed uniform")?.to_string();
        if name.ends_with(';') {
            name.truncate(name.len() - 1);
        }
        result.push(Uniform {
            name: name.parse().unwrap(),
            u_type,
        });
    }
    Ok(result)
}

fn make_parameter(uniform: &Uniform) -> proc_macro2::TokenStream {
    let name = &uniform.name;
    let type_token = get_type_token(&uniform.u_type);
    quote! {
        #name: #type_token
    }
}

fn generate_expression(struct_name: Ident, uniforms: Vec<Uniform>) -> proc_macro2::TokenStream {
    let struct_member_uniforms = uniforms
        .iter()
        .filter(|u| UniformType::Sampler2D.ne(&u.u_type));
    let struct_members: Vec<proc_macro2::TokenStream> =
        struct_member_uniforms.clone().map(make_parameter).collect();
    let fn_arguments: Vec<proc_macro2::TokenStream> = uniforms.iter().map(make_parameter).collect();
    let member_initializers: Vec<proc_macro2::TokenStream> = struct_member_uniforms
        .map(|u| {
            let name = &u.name;
            quote! {
                #name: Default::default()
            }
        })
        .collect();

    let mut setup_steps =  Vec::new();
    let mut update_steps= Vec::new();
    for (idx, elem) in uniforms.iter().enumerate() {
        if let Some(setup) = make_setup_step(idx as i32, elem) {
            setup_steps.push(setup);
        }
        update_steps.push(make_update_step(idx as i32, elem));
    }
    let generated = quote! {
        struct #struct_name {
            program: ::utility::Program,
            #(#struct_members),*
        }

        impl #struct_name {
            pub fn create(
                context: &::web_sys::WebGl2RenderingContext,
                program: ::utility::Program
            ) -> Self {
                context.use_program(Some(program.program()));
                #(#setup_steps);*;
                Self {
                    program,
                    #(#member_initializers),*
                }
            }
            pub fn set_arguments(
                &mut self,
                context: &::web_sys::WebGl2RenderingContext,
                #(#fn_arguments),*
            ) -> () {
                context.use_program(Some(self.program.program()));
                #(#update_steps);*;
            }
        }
    };
    generated.into()
}

#[proc_macro]
#[proc_macro_error]
pub fn render_pipeline(input: TokenStream) -> TokenStream {
    let span = Span::call_site();
    let mut iter = input.clone().into_iter();
    if iter.clone().count() != 3 {
        abort!(
            span,
            "arguments must be of the form <struct_name>, \"<shader_path>\""
        );
    }
    let struct_name = match iter.next().unwrap() {
        TokenTree::Ident(ident) => Ident::new(&ident.to_string(), ident.span().into()),
        _ => abort!(span, "first value must be rust identifier"),
    };

    match iter.next().unwrap() {
        TokenTree::Punct(punct) if punct.as_char() == ',' => (),
        _ => abort!(span, "punctuation must be ','"),
    };

    let shader_path = match iter.next().unwrap() {
        TokenTree::Literal(literal) => literal.to_string(),
        _ => abort!(span, "shader path must be a string literal"),
    };


    let uniforms = parse_shader_path(&shader_path, &span)
        .map_err(|err| {
            abort!(
                Span::call_site(),
                format!("failed to parse file {}: {}", shader_path.replace('\"', "'"), err)
            )
        })
        .unwrap();

    generate_expression(struct_name, uniforms).into()
}

