precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_target;
uniform sampler2D u_velocity;
uniform vec2 u_texel_size;
uniform float u_timestep;

void main() {
    vec2 pos =
        v_texcoord -
        u_timestep * u_texel_size *
        texture2D(u_velocity, v_texcoord).rg;
    gl_FragColor = texture2D(u_target, pos);
}