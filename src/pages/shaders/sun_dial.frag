#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform vec2 u_sun_location;

const vec3 BLUE = vec3(0.0, 0.0, 1.0);
const vec3 GREEN = vec3(0.0, 1.0, 0.0);

void main() {
    vec3 color = BLUE;
    if (length(u_sun_location - v_texcoord) < 0.2) {
        color = GREEN;
    }
    fragColor = vec4(color, 1.0);
}
