precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_texture;

void main() {
    gl_FragColor =  1000.0 * texture2D(u_texture, v_texcoord);
}