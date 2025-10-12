precision mediump float;

varying vec2 v_texcoord;

uniform vec2 u_direction;
uniform vec2 u_texel_size;
uniform float u_wind_speed;
uniform float u_max_height;
uniform sampler2D u_sand;
uniform sampler2D u_random;
uniform sampler2D u_shadow;
uniform float u_pickup_rate;


float round(in float value) {
    return floor(value + 0.5);
}

vec4 tex_bilerp(in sampler2D tex, in vec2 uv, in vec2 tsize) {
    vec2 st = uv / tsize - 0.5;

    vec2 iuv = floor(st);
    vec2 fuv = fract(st);

    vec4 a = texture2D(tex, (iuv + vec2(0.5, 0.5)) * tsize);
    vec4 b = texture2D(tex, (iuv + vec2(1.5, 0.5)) * tsize);
    vec4 c = texture2D(tex, (iuv + vec2(0.5, 1.5)) * tsize);
    vec4 d = texture2D(tex, (iuv + vec2(1.5, 1.5)) * tsize);

    return mix(mix(a, b, fuv.x), mix(c, d, fuv.x), fuv.y);
}

void main() {
    float new_sand = tex_bilerp(u_sand, v_texcoord + normalize(u_direction) * u_wind_speed, u_texel_size).y * u_max_height;
    float base_sand = round(texture2D(u_sand, v_texcoord).x * u_max_height);
    float shadowed = texture2D(u_shadow, v_texcoord).x;
    float random = texture2D(u_random, v_texcoord).x;
    if (shadowed > 0.5) {
        if (random < u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    } else if (base_sand == 0.0) {
        if (random < 0.4 * u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    } else {
        if (random < 0.6 * u_pickup_rate) {
            base_sand += new_sand;
            new_sand = 0.0;
        }
    }

    if (shadowed < 0.5) {
        if (random < u_pickup_rate && base_sand > 0.0) {
            new_sand += 1.0;
            base_sand -= 1.0;
        }
    }

    gl_FragColor = vec4(base_sand / u_max_height, new_sand / u_max_height, 0.0, 0.0);
}