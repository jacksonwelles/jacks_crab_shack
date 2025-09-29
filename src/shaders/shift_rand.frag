precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_texture;
uniform vec2 u_texel_size;
uniform vec2 u_direction;

void main() {
    gl_FragColor = texture2D(u_texture, v_texcoord + u_direction * u_texel_size);
}
