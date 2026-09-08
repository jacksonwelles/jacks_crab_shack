#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform vec2 u_texel_size;

vec2 get_direction(in int idx) {
    if (idx == 0) { return vec2( 0.0,  1.0); }
    if (idx == 1) { return vec2( 1.0,  1.0); }
    if (idx == 2) { return vec2( 1.0,  0.0); }
    if (idx == 3) { return vec2( 1.0, -1.0); }
    if (idx == 4) { return vec2( 0.0, -1.0); }
    if (idx == 5) { return vec2(-1.0, -1.0); }
    if (idx == 6) { return vec2(-1.0,  0.0); }
    if (idx == 7) { return vec2(-1.0,  1.0); }
    return vec2(0.0, 0.0);
}

float new_height() {
    float sand = texture(u_sand, v_texcoord).r * u_max_height;
    vec2 directions[8] = vec2[8](
        vec2( 0.0,  1.0),
        vec2( 1.0,  1.0),
        vec2( 1.0,  0.0),
        vec2( 1.0, -1.0),
        vec2( 0.0, -1.0),
        vec2(-1.0, -1.0),
        vec2(-1.0,  0.0),
        vec2(-1.0,  1.0)
    );

    float change_in_sand = 0.0;

    for (int i = 0; i < 8; i++) {
        float neighbor_sand = texture(
            u_sand,
            v_texcoord + directions[i] * u_texel_size
        ).r * u_max_height;
        float diff = neighbor_sand - sand;
        float modifier = 1.0f;
        if (mod(float(i), 2.0) == 1.0) {
            modifier = 0.2f;
        }
        if (diff > 2.0) {
            change_in_sand += 0.25 * modifier;
        }
        if (diff < -2.0) {
            change_in_sand -= 0.25 * modifier;
        }
    }
    return (sand + change_in_sand) / u_max_height;
}

void main() {
    fragColor = vec4(new_height(), texture(u_sand, v_texcoord).y, 0.0, 0.0);
}
