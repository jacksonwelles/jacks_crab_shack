#version 300 es

precision mediump float;

in vec2 v_texcoord;

out vec4 frag_color;

uniform sampler2D u_solution;
uniform sampler2D u_initial;

uniform vec2 u_texel_size;
uniform vec2 u_alpha;
uniform vec2 u_r_beta;

void main() {
    vec4 s_l = texture(u_solution, v_texcoord - vec2(u_texel_size.x, 0));
    vec4 s_r = texture(u_solution, v_texcoord + vec2(u_texel_size.x, 0));
    vec4 s_b = texture(u_solution, v_texcoord - vec2(0, u_texel_size.y));
    vec4 s_t = texture(u_solution, v_texcoord + vec2(0, u_texel_size.y));
    vec4 iv = texture(u_initial, v_texcoord);

    frag_color =
        (s_l + s_r + s_b + s_t + vec4(u_alpha.x, u_alpha.y , 0, 0) * iv) *
        vec4(u_r_beta.x, u_r_beta.y, 0, 0);
}