#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_sand;
uniform sampler2D u_shadow;
uniform vec3 u_voxel_size;
uniform vec3 u_direction;

const vec3 GROUND = vec3(0.759, 0.6, 0.475);
const vec3 SAND = vec3(0.878, 0.668, 0.365);
const vec3 SUN = vec3(1.0, 1.0, 1.0);
const vec3 SKY = SUN; // vec3(0.7, 0.9, 1.0);

vec3 get_normal() {
    float up    = texture(u_sand, v_texcoord + vec2( 0.0,  1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float down  = texture(u_sand, v_texcoord + vec2( 0.0, -1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float left  = texture(u_sand, v_texcoord + vec2(-1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float right = texture(u_sand, v_texcoord + vec2( 1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;

    vec3 a = vec3(1.0, 0.0, right - left);
    vec3 b = vec3(0.0, 1.0, up - down);

    return normalize(cross(a, b));
}


void main() {
    float shadow = texture(u_shadow, v_texcoord).r;
    float ground_sand = texture(u_sand, v_texcoord).x;
    float windborn_sand = texture(u_sand, v_texcoord).y;
    float diffuse = max(0.0, dot(normalize(u_direction), get_normal()));

    vec3 base = SAND;
    if (ground_sand == 0.0) {
        base = GROUND;
    }
    base = base * (SKY * 0.5 + SUN * 0.8 * diffuse * (1.0 -  shadow));
    fragColor = vec4(base.xyz, 1.0); //mix(base, SAND, windborn_sand * 64.0);
}
