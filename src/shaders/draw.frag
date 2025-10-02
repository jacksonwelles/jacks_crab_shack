precision highp float;

varying vec2 v_texcoord;

uniform sampler2D u_sand;
uniform sampler2D u_shadow;
uniform vec3 u_voxel_size;
uniform vec3 u_direction;

const vec4 DARK = vec4(0.502, 0.467, 0.361, 1.0);
const vec4 LIGHT = vec4(0.796, 0.741, 0.576, 1.0);

vec3 get_normal() {
    float up    = texture2D(u_sand, v_texcoord + vec2( 0.0,  1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float down  = texture2D(u_sand, v_texcoord + vec2( 0.0, -1.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float left  = texture2D(u_sand, v_texcoord + vec2(-1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;
    float right = texture2D(u_sand, v_texcoord + vec2( 1.0,  0.0 ) * u_voxel_size.xy ).r / u_voxel_size.z;

    vec3 a = vec3(2.0, 0.0, right - left);
    vec3 b = vec3(0.0, 2.0, up - down);

    return normalize(cross(a, b));
}





void main() {
    float shadow = texture2D(u_shadow, v_texcoord).r;

    float diffuse = max(0.0, dot(normalize(u_direction), get_normal()));

    gl_FragColor = DARK + (LIGHT - DARK) * diffuse * (1.0-shadow);
}
