#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec2 fragColor;

uniform sampler2D u_sand;
uniform vec2 u_texel_size;
uniform float u_max_height;
uniform float u_radius;
uniform vec2 u_center;

void main() {
    vec4 base_tex = texture(u_sand, v_texcoord);
    float base = base_tex.r;
    float d_texels = length((v_texcoord - u_center) / u_texel_size);
    float sigma = u_radius / 5.0;
    // float sand_to_add = 0.0;
    // if (d_texels < u_radius * 0.7) {
    float sand_to_add = 50.0 * (0.3989 / sigma)  * exp(-(d_texels * d_texels) / (2.0 * sigma * sigma) );
    // }

    fragColor = vec2(
        min(
            1.0,
            base + sand_to_add / u_max_height
        ),
        base_tex.y
    );
}
