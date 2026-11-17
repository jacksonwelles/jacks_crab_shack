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

void deposit(inout float base, inout float windborn, in float amt) {
    base += min(windborn, amt);
    windborn -= min(windborn, amt);
}

void lift(inout float base, inout float windborn, in float amt) {
    base -= min(base, amt);
    windborn += min(base, amt);
}

void main() {
    float windborn_sand = tex_bilerp(u_sand, v_texcoord + normalize(u_direction) * u_wind_speed, u_texel_size).y;
    float base_sand = texture(u_sand, v_texcoord).x;
    float shadowed = texture(u_shadow, v_texcoord).x;
    float rand = texture(u_random, v_texcoord).x;
    float low_wind_height = 0.9 / u_max_height;
    float low_wind_rate = 0.05 * u_pickup_rate;
    float low_wind_deposit = 0.02 / u_max_height;

    float deposit_rate = 0.6 * u_pickup_rate;
    float normal_deposit = windborn_sand; // 0.5 / u_max_height;
    float shadow_deposit = windborn_sand; //10.0 / u_max_height;
    float shadow_deposit_rate = u_pickup_rate;

    float normal_lift = 0.5 / u_max_height;
    float lift_rate = 0.7 * u_pickup_rate;
    bool in_shadow = shadowed > 0.5;

    float deposit_rand = rand;
    float lift_rand = mod(rand, 0.0625) * 16.0;
    if (in_shadow) {
        if (deposit_rand > shadow_deposit_rate) {
            deposit(base_sand, windborn_sand, shadow_deposit);
        }
    } else if (base_sand < low_wind_height) {
        if (deposit_rand < low_wind_rate) {
            deposit(base_sand, windborn_sand, low_wind_deposit);
        }
    } else /* height above cutoff*/ {
        if (deposit_rand < deposit_rate) {
            deposit(base_sand, windborn_sand, normal_deposit);
        }
    }
    if (!in_shadow && base_sand > low_wind_height) {
        if (lift_rand < lift_rate) {
            lift(base_sand, windborn_sand, normal_lift);
            if (base_sand <= low_wind_height) {
                lift(base_sand, windborn_sand, base_sand);
            }
        }
    }
    fragColor = vec4(base_sand, windborn_sand, 0.0, 0.0);
}