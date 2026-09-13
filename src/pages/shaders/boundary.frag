#version 300 es

precision mediump float;

in vec2 v_texcoord;

out vec4 frag_color;

uniform sampler2D u_target;
uniform sampler2D u_boundary_offsets;
uniform vec2 u_texel_size;
uniform float u_scale;


void main()
{
    float scale = u_scale;
    vec2 offset = texture(u_boundary_offsets, v_texcoord).rg * u_texel_size;

    // don't scale if there's no boundary offset
    if (offset == vec2(0,0)) {
        scale = 1.0;
    }

    frag_color = scale * texture(u_target, v_texcoord + offset);
}