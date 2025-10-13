#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform vec2 u_direction;
uniform vec2 u_texel_size;
uniform float u_wind_speed;
uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_random;
uniform sampler2D u_shadow;
uniform float u_pickup_rate;


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

float hash(in vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

void main() {
    float new_sand = tex_bilerp(u_sand, v_texcoord + normalize(u_direction) * u_wind_speed, u_texel_size).y;
    float base_sand = texture(u_sand, v_texcoord).x;
    float shadowed = texture(u_shadow, v_texcoord).x;
    float rand = texture(u_random, v_texcoord).x;
    if (shadowed > 0.5) {
        if (rand < u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    } else if (base_sand < 1.0 / u_max_height) {
        if (rand < 0.4 * u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    } else {
        if (rand < 0.6 * u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    }
    if (shadowed < 0.5) {
        if (rand < u_pickup_rate) {
            new_sand = min (1.0, new_sand + min(base_sand, 1.0 / u_max_height));
            base_sand = max(0.0, base_sand - 1.0/u_max_height);
        }
    }
    fragColor = vec4(base_sand, new_sand, 0.0, 0.0);
}