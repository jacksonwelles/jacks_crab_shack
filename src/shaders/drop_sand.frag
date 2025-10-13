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
    if (length((v_texcoord - u_center) / u_texel_size) < u_radius ) {
        fragColor = vec4(
            min(
                1.0,
                base + 1.0 / u_max_height
            )
        , 0.0, 0.0, 0.0);
    } else {
        fragColor = vec4(base, base_tex.y, 0.0, 0.0);
    }
}
