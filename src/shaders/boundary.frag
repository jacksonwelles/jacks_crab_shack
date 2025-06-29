precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_target;
uniform sampler2D u_boundary_offsets;
uniform vec2 u_texel_size;
uniform float u_scale;


void main()
{
    float scale = u_scale;
    vec2 offset = texture2D(u_boundary_offsets, v_texcoord).rg * u_texel_size;

    // don't scale if there's no boundary offset
    if (offset == vec2(0,0)) {
        scale = 1.0;
    }

    gl_FragColor = scale * texture2D(u_target, v_texcoord + offset);
}