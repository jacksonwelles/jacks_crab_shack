#version 300 es

precision highp float;

in vec2 v_texcoord;

out vec4 frag_color;

uniform sampler2D u_target;
uniform sampler2D u_velocity;
uniform vec2 u_target_texel_size;
uniform vec2 u_velocity_texel_size;
uniform float u_timestep;

vec4 tex_bilerp(in sampler2D tex, in vec2 uv, in vec2 tsize) {
    vec2 st = uv / tsize - 0.5;

    vec2 iuv = floor(st);
    vec2 fuv = fract(st);

    vec4 a = texture(tex, (iuv + vec2(0.5, 0.5)) * tsize);
    vec4 b = texture(tex, (iuv + vec2(1.5, 0.5)) * tsize);
    vec4 c = texture(tex, (iuv + vec2(0.5, 1.5)) * tsize);
    vec4 d = texture(tex, (iuv + vec2(1.5, 1.5)) * tsize);

    return mix(mix(a, b, fuv.x), mix(c, d, fuv.x), fuv.y);
}

void main() {
    vec2 pos =
        v_texcoord -
        u_timestep * u_velocity_texel_size *
        tex_bilerp(u_velocity, v_texcoord, u_velocity_texel_size).rg;
    frag_color = tex_bilerp(u_target, pos, u_target_texel_size);
}
