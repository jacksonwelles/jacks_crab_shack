#version 300 es

precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_sand;
uniform vec2 u_texel_size;
uniform float u_max_height;
uniform float u_radius;
uniform vec2 u_center;

void main() {
    vec4 base_tex = texture(u_sand, v_texcoord);
    float base = base_tex.r;
    float d_texels = length((v_texcoord - u_center) / u_texel_size);
    // float sand_to_add = 1.0;
    // if (d_texels > u_radius) {
    //     sand_to_add = 0.0;
    // }
    // fragColor = vec4(
    //     min(
    //         1.0,
    //         base + sand_to_add / u_max_height
    //     )
    // , 0.0, 0.0, 0.0);
    float sigma = u_radius / 5.0;
    float sand = 50.0 * (0.3989 / sigma)  * exp(-(d_texels * d_texels) / (2.0 * sigma * sigma) );
    fragColor = vec4(
        min(
            1.0,
            base + sand / u_max_height
        )
    , 0.0, 0.0, 0.0);
}
