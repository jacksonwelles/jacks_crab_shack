precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_velocity;
uniform vec2 u_location;
uniform vec2 u_direction;
uniform float u_scale;
uniform float u_radius;


void main()
{
    float dist = distance(u_location, v_texcoord);
    if (dist < u_radius) {
        gl_FragColor =
            texture2D(u_velocity, v_texcoord) +
            vec4(u_direction * u_scale * ((u_radius - dist)/u_radius), 0, 0);
    } else {
        gl_FragColor = texture2D(u_velocity, v_texcoord);
    }
}