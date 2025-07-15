precision mediump float;

varying vec2 v_texcoord;

uniform float u_max_height;
uniform sampler2D u_sand;
uniform vec2 u_texel_size;

float adjust(float sand, float neighbor) {
    float diff = u_max_height * (sand - neighbor);
    if (diff > 2.0) {
        return -1.0;
    }
    if (diff < -2.0) {
        return 1.0;
    }
    return 0.0;
}

void main() {
    float sand = texture2D(u_sand, v_texcoord).r;
    float change = 0.0;
    change += adjust(sand, texture2D(u_sand, v_texcoord + vec2( 1.0,  0.0) * u_texel_size).r);
    change += adjust(sand, texture2D(u_sand, v_texcoord + vec2(-1.0,  0.0) * u_texel_size).r);
    change += adjust(sand, texture2D(u_sand, v_texcoord + vec2( 0.0,  1.0) * u_texel_size).r);
    change += adjust(sand, texture2D(u_sand, v_texcoord + vec2( 0.0, -1.0) * u_texel_size).r);
    change = min(change, 1.0);
    change = max(change, -1.0);
    gl_FragColor = vec4(sand + change / u_max_height, 0.0, 0.0, 0.0);
}
