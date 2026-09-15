#version 300 es

precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_random;
uniform vec2 u_texel_size;


float movement_direction(float rand) {
    float sand = texture(u_sand, v_texcoord).r;

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
    bool can_accept_sand[8] = bool[8](
        false, false, false, false, false, false, false, false
    );
    float acceptors = 0.0;
    for (int i = 0; i < 8; i++) {
        float neighbor = texture(
            u_sand,
            v_texcoord + directions[i] * u_texel_size
        ).r;
        if ((sand - neighbor) * u_max_height > 2.0) {
            can_accept_sand[i] = true;
            acceptors += 1.0;
        } else {
            can_accept_sand[i] = false;
        }
    }
    if (acceptors == 0.0) {
        return 1.0;
    }
    if (acceptors == 1.0){
        return 1.0;
    }
    int selection = int(round(rand * (acceptors - 1.0)));
    for (int i = 0; i < 8; i++) {
        if (can_accept_sand[i] && selection-- == 0) {
            return float(i) / 8.0;
        }
    }
    return 1.0;
}

void main() {
    float movement_rand = texture(u_random, v_texcoord).x;
    float amt_rand = mod(movement_rand, 0.125) * 8.0;
    float amt = amt_rand * 0.05 + 0.975;
    fragColor = vec4(movement_direction(movement_rand), amt / u_max_height, 0.0, 0.0);
}
