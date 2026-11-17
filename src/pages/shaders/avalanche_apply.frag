#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_delta;
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
    float sand = texture(u_sand, v_texcoord).r;
    vec4 local_delta = texture(u_delta, v_texcoord);
    int move_dir = int(round(local_delta.r * 8.0));
    float move_amount = local_delta.g;

    float change_in_sand = 0.0;
    if (move_dir != 8) {
        change_in_sand -= move_amount;
    }

    for (int i = 0; i < 8; i++) {
        vec4 neighbor_delta = texture(
            u_delta,
            v_texcoord + get_direction(i) * u_texel_size
        );
        int neighbor_dir = int(round(neighbor_delta.r * 8.0));
        float neighbor_amount = neighbor_delta.g;
        if (neighbor_dir == 8) {
            continue;
        }
        if (int(mod(float(neighbor_dir + 4), 8.0)) == i) {
            change_in_sand += neighbor_amount;
        }
    }
    return min(u_max_height, sand + change_in_sand);
}

void main() {
    fragColor = vec4(new_height(), texture(u_sand, v_texcoord).y, 0.0, 0.0);
}
