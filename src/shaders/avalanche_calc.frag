#version 300 es

precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_random;
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

void update_accept(inout bool accept[8], in int idx, in bool value) {
    if (idx == 0) { accept[0] = value; return; }
    if (idx == 1) { accept[1] = value; return; }
    if (idx == 2) { accept[2] = value; return; }
    if (idx == 3) { accept[3] = value; return; }
    if (idx == 4) { accept[4] = value; return; }
    if (idx == 5) { accept[5] = value; return; }
    if (idx == 6) { accept[6] = value; return; }
    if (idx == 7) { accept[7] = value; return; }
}

bool get_accept(in bool accept[8], in int idx) {
    if (idx == 0) { return accept[0]; }
    if (idx == 1) { return accept[1]; }
    if (idx == 2) { return accept[2]; }
    if (idx == 3) { return accept[3]; }
    if (idx == 4) { return accept[4]; }
    if (idx == 5) { return accept[5]; }
    if (idx == 6) { return accept[6]; }
    if (idx == 7) { return accept[7]; }
}

float movement_direction() {
    float sand = texture(u_sand, v_texcoord).r;
    float rand = round(texture(u_random, v_texcoord).x * 8.0);

    bool can_accept_sand[8];
    int acceptors = 0;
    for (int i = 0; i < 8; i++) {
        float neighbor = texture(
            u_sand,
            v_texcoord + get_direction(i) * u_texel_size
        ).r;
        if (sand - neighbor > 2.0 / u_max_height) {
            update_accept(can_accept_sand, i, true);
            acceptors++;
        } else {
            update_accept(can_accept_sand, i, false);
        }
    }
    if (acceptors == 0) {
        return 1.0;
    }
    int selection = int(mod(rand, float(acceptors)));
    for (int i = 0; i < 8; i++) {
        if (get_accept(can_accept_sand, i) && selection-- == 0) {
            return float(i) / 8.0;
        }
    }
    return 1.0;
}

void main() {
    fragColor = vec4(movement_direction(), 0.0, 0.0, 0.0);
}
