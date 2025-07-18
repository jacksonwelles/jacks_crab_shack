use proc_macro::{TokenStream, TokenTree};
use proc_macro_error::{abort, proc_macro_error};
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Type};

enum UniformType {
    Sampler2D,
    Float,
    Vec2,
    Vec3,
    Vec4,
}

struct Uniform {
    name: String,
    u_type: UniformType,
}

fn get_uniform_type(type_name: &str) -> Result<UniformType, String> {
    match type_name {
        "Sampler2D" => Ok(UniformType::Sampler2D),
        "float" => Ok(UniformType::Float),
        "vec2" => Ok(UniformType::Vec2),
        "vec3" => Ok(UniformType::Vec3),
        "vec4" => Ok(UniformType::Vec4),
        _ => Err(format!("could not parse type: '{}'", type_name)),
    }
}

fn parse_shader_source(source: &str) -> Result<Vec<Uniform>, String> {
    let mut split = source.split(' ');
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
        result.push(Uniform { name, u_type });
    }
    Ok(result)
}

fn make_struct_member(uniform: &Uniform) -> proc_macro2::TokenStream {
    let name = Ident::new(&uniform.name, Span::call_site());
    let u_type: proc_macro2::TokenStream =
        match uniform.u_type {
            UniformType::Sampler2D => "String",
            UniformType::Float => "f32",
            UniformType::Vec2 => "(f32, f32)",
            UniformType::Vec3 => "(f32, f32, f32)",
            UniformType::Vec4 => "(f32, f32, f32, f32)",
        }.parse().unwrap();
    quote! {
        #name: #u_type
    }
}

#[proc_macro]
#[proc_macro_error]
pub fn render_pipeline(input: TokenStream) -> TokenStream {
    let span = Span::call_site();
    let mut iter = input.clone().into_iter();
    if iter.clone().count() != 3 {
        abort!(
            span,
            "arguments must be of the form <struct_name>, \"<shader_source>\""
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

    let shader_source = match iter.next().unwrap() {
        TokenTree::Literal(literal) => literal.to_string(),
        _ => abort!(span, "shader source must be a string literal"),
    };

    if !shader_source.starts_with('"') || !shader_source.ends_with('"') {
        abort!(span, "shader source must be a string literal");
    }
    let unquoted_src = &shader_source[1..(shader_source.len() - 1)];

    let uniforms = parse_shader_source(unquoted_src)
        .map_err(|err| {
            abort!(
                Span::call_site(),
                format!("failed to parse source: {}", err)
            )
        })
        .unwrap();
    let members: Vec<proc_macro2::TokenStream> = uniforms.iter().map(make_struct_member).collect();
    let generated = quote! {
        struct #struct_name {
            #(#members),*
        }
    };
    generated.into()
}
