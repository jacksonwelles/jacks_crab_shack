#version 300 es

precision mediump float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_random;
uniform vec2 u_texel_size;


float movement_direction() {
    float sand = texture(u_sand, v_texcoord).r;
    float rand = texture(u_random, v_texcoord).x;

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
    float diagonal_chance = 0.0;
    for (int i = 0; i < 8; i++) {
        float neighbor = texture(
            u_sand,
            v_texcoord + directions[i] * u_texel_size
        ).r;
        // if ((i & 1) == 1) {
        //     continue;
        // }
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
    float remainder = rand;
    float adjusted_rand = rand * (acceptors - 1.0);
    float rounded = round(adjusted_rand);
    if (acceptors > 1.0) {
        remainder = adjusted_rand - rounded;
    }
    int selection = int(rounded);
    for (int i = 0; i < 8; i++) {
        if (can_accept_sand[i] && selection-- == 0) {
            if (remainder > diagonal_chance) {
                return 1.0;
            }
            return float(i) / 8.0;
        }
    }
    return 1.0;
}

void main() {
    fragColor = vec4(movement_direction(), 0.0, 0.0, 0.0);
}
