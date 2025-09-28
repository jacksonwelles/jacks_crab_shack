precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_sand;
uniform float u_layer;
uniform vec2 u_texel_size;
uniform vec2 u_direction;
uniform float u_tan_theta;
uniform float u_max_height;

const vec4 DARK = vec4(0.502, 0.467, 0.361, 1.0);
const vec4 LIGHT = vec4(0.796, 0.741, 0.576, 1.0);

float bilerp_with_threshhold(in sampler2D tex, in vec2 uv, in float threshold) {

    vec2 st = uv / u_texel_size - 0.5;

    vec2 iuv = floor(st);
    vec2 fuv = fract(st);

    float a = texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_texel_size).r;
    float b = texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_texel_size).r;
    float c = texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_texel_size).r;
    float d = texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_texel_size).r;

    vec4 result = vec4(a,b,c,d);
    result = min(vec4(1.0, 1.0, 1.0, 1.0), floor(result / threshold));

    /*
    float a,b,c,d;

    if (u_layer == 0.0) {
        a = floor(texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_texel_size).r * r_threshold);
        b = floor(texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_texel_size).r * r_threshold);
        c = floor(texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_texel_size).r * r_threshold);
        d = floor(texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_texel_size).r * r_threshold);
    } else if (u_layer == 1.0) {
        a = floor(texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_texel_size).g * r_threshold);
        b = floor(texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_texel_size).g * r_threshold);
        c = floor(texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_texel_size).g * r_threshold);
        d = floor(texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_texel_size).g * r_threshold);
    } else if (u_layer == 2.0) {
        a = floor(texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_texel_size).b * r_threshold);
        b = floor(texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_texel_size).b * r_threshold);
        c = floor(texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_texel_size).b * r_threshold);
        d = floor(texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_texel_size).b * r_threshold);
    } else {
        a = floor(texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_texel_size).a * r_threshold);
        b = floor(texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_texel_size).a * r_threshold);
        c = floor(texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_texel_size).a * r_threshold);
        d = floor(texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_texel_size).a * r_threshold);
    }
    */

    return min(mix(mix(result[0], result[1], fuv.x), mix(result[2], result[3], fuv.x), fuv.y), 1.0);
}


void main() {
    float base_height = texture2D(u_sand, v_texcoord).r;
    float px_tan_theta = u_tan_theta / u_max_height;
    vec2 dir = normalize(u_direction) * u_texel_size;
    vec2 pos = v_texcoord;
    float threshold = base_height + px_tan_theta * 0.5;
    float shadowed = 0.0;
    for (int i = 0; i < 4096; i ++) {
        pos += dir;
        threshold += px_tan_theta;
        if(threshold > 1.0) {
            break;
        }
        shadowed = max(shadowed, bilerp_with_threshhold(u_sand, pos, threshold));
        if (shadowed >= 0.99) {
            gl_FragColor = DARK;
            return;
        }
    }
    gl_FragColor = LIGHT + (DARK - LIGHT) * shadowed;
}
