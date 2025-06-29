precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_velocity;
uniform vec2 u_texel_size;

void main()
{
    vec4 v_l = texture2D(u_velocity, v_texcoord - vec2(u_texel_size.x, 0));
    vec4 v_r = texture2D(u_velocity, v_texcoord + vec2(u_texel_size.x, 0));
    vec4 v_b = texture2D(u_velocity, v_texcoord - vec2(0, u_texel_size.y));
    vec4 v_t = texture2D(u_velocity, v_texcoord + vec2(0, u_texel_size.y));

    gl_FragColor = vec4(0.5 * (
        u_texel_size.x * (v_r.x - v_l.x) +
        u_texel_size.y * (v_t.y - v_b.y)
    ),0,0,0);
}