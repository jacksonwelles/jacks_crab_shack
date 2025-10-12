precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_sand;
uniform vec2 u_texel_size;
uniform float u_max_height;
uniform float u_radius;
uniform vec2 u_center;

void main() {
    vec4 base_tex = texture2D(u_sand, v_texcoord);
    float base = base_tex.r;
    if (length((v_texcoord - u_center) / u_texel_size) < u_radius ) {
        gl_FragColor = vec4(
            min(
                u_max_height,
                u_max_height * base + 1.0
            ) / u_max_height
        , 0.0, 0.0, 0.0);
    } else {
        gl_FragColor = vec4(base, base_tex.y, 0.0, 0.0);
    }
}
