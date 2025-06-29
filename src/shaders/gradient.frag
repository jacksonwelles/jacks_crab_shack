precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_velocity;
uniform sampler2D u_pressure;

uniform vec2 u_texel_size;

void main()
{
    float p_l = texture2D(u_pressure, v_texcoord - vec2(u_texel_size.x, 0)).x;
    float p_r = texture2D(u_pressure, v_texcoord + vec2(u_texel_size.x, 0)).x;
    float p_b = texture2D(u_pressure, v_texcoord - vec2(0, u_texel_size.y)).x;
    float p_t = texture2D(u_pressure, v_texcoord + vec2(0, u_texel_size.y)).x;

    gl_FragColor =
        texture2D(u_velocity, v_texcoord) -
        vec4(0.5 * u_texel_size * vec2(p_r - p_l, p_t- p_b), 0, 0);
}