precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_velocity;
uniform sampler2D u_pressure;

uniform vec2 u_texel_size;

void main()
{
    float p_l = texture(u_pressure, frag_uv - vec2(u_texel_size.x, 0)).x;
    float p_r = texture(u_pressure, frag_uv + vec2(u_texel_size.x, 0)).x;
    float p_b = texture(u_pressure, frag_uv - vec2(0, u_texel_size.y)).x;
    float p_t = texture(u_pressure, frag_uv + vec2(0, u_texel_size.y)).x;

    gl_FragColor =
        texture(u_velocity, frag_uv) -
        vec4(0.5 * u_texel_size * vec2(p_r - p_l, p_t- p_b), 0, 0);
}