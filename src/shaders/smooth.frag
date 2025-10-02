precision highp float;

varying vec2 v_texcoord;

uniform sampler2D u_sand;
uniform vec2 u_texel_size;

void main() {
    float total = texture2D(u_sand, v_texcoord + u_texel_size * vec2(0.0,0.0)).r;

    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2( 0.0,  1.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2( 1.0,  1.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2( 1.0,  0.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2( 1.0, -1.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2( 0.0, -1.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2(-1.0, -1.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2(-1.0,  0.0)).r;
    total += texture2D(u_sand, v_texcoord + u_texel_size * vec2(-1.0,  1.0)).r;

    gl_FragColor = vec4(total / 9.0, 0.0, 0.0, 0.0);
}
