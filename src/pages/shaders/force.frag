#version 300 es

precision mediump float;

in vec2 v_texcoord;


out vec4 frag_color;

uniform sampler2D u_velocity;
uniform vec2 u_location;
uniform vec2 u_direction;
uniform float u_scale;
uniform float u_radius;


void main()
{
    float dist = distance(u_location, v_texcoord);
    if (dist < u_radius) {
        frag_color =
            texture(u_velocity, v_texcoord) +
            vec4(u_direction * u_scale * ((u_radius - dist)/u_radius), 0, 0);
    } else {
        frag_color = texture(u_velocity, v_texcoord);
    }
}