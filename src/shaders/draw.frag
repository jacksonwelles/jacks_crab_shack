#version 300 es

precision highp float;

in vec2 v_texcoord;
out vec4 fragColor;

uniform sampler2D u_sand;
uniform sampler2D u_shadow;
uniform vec3 u_voxel_size;
uniform vec3 u_direction;

const vec4 DARK = vec4(0.502, 0.467, 0.361, 1.0);
const vec4 LIGHT = vec4(0.796, 0.741, 0.576, 1.0);
// const vec4 AIR = vec4(0.918, 0.8, 0.737, 1.0);
const vec4 GROUND = vec4(0.411, 0.435, 0.435, 1.0);

vec3 get_normal() {
    float up    = texture(u_sand, v_texcoord + vec2( 0.0,  1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float down  = texture(u_sand, v_texcoord + vec2( 0.0, -1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float left  = texture(u_sand, v_texcoord + vec2(-1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float right = texture(u_sand, v_texcoord + vec2( 1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;

    vec3 a = vec3(2.0, 0.0, right - left);
    vec3 b = vec3(0.0, 2.0, up - down);

    return normalize(cross(a, b));
}


void main() {
    float shadow = texture(u_shadow, v_texcoord).r;
    float ground_sand = texture(u_sand, v_texcoord).x;
    float windborn_sand = texture(u_sand, v_texcoord).y;
    float diffuse = max(0.0, dot(normalize(u_direction), get_normal()));

    vec4 base = DARK + (LIGHT - DARK) * diffuse * (1.0-shadow);
    if (ground_sand == 0.0) {
        base = GROUND;
    }
    fragColor = mix(base, LIGHT, windborn_sand * 64.0);
}
