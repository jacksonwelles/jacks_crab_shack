#version 300 es

precision mediump float;

in vec2 v_texcoord;


out vec4 frag_color;

uniform sampler2D u_texture;

void main() {
    frag_color =  1000.0 * texture(u_texture, v_texcoord);
}