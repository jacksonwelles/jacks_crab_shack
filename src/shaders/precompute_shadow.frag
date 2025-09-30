precision mediump float;

varying vec2 v_texcoord;

uniform sampler2D u_texture;
uniform float u_factor;
uniform vec2 u_direction;
uniform vec2 u_texel_size;
uniform float u_pass_num;

vec4 get_max_height() {
    vec4 max_height = vec4(0.0, 0.0, 0.0, 0.0);
    float step_length = pow(u_factor, u_pass_num);
    vec2 norm_direction = normalize(u_direction) * u_texel_size;
    for (int i = 0; i < 20; i++) {
        if (i == int(u_factor)) {
            break;
        }
        vec4 height = texture2D(u_texture, v_texcoord + norm_direction * float(i) * step_length);
        max_height = max(height, max_height);
    }
    return max_height;
}

void main() {
    vec4 max_height = get_max_height();
    if (u_pass_num == 0.0) {
        gl_FragColor = vec4(max_height[0], 0.0, 0.0, 0.0);
        return;
    } else if (u_pass_num == 1.0) {
        vec4 prev = texture2D(u_texture, v_texcoord);
        gl_FragColor = vec4(prev.x, max_height.x, 0.0, 0.0);
        return;
    } else if (u_pass_num == 2.0) {
        vec4 prev = texture2D(u_texture, v_texcoord);
        gl_FragColor = vec4(prev.x, prev.y, max_height.y, 0.0);
        return;
    } else if (u_pass_num == 3.0) {
        vec4 prev = texture2D(u_texture, v_texcoord);
        gl_FragColor = vec4(prev.x, prev.y, prev.z, max_height.z);
        return;
    }
}
