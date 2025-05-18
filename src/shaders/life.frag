precision mediump float;

varying vec2 v_texcoord;
uniform sampler2D u_texture;
uniform vec2 u_texel_size;

void main() {
    int sum = 0;
    bool alive = texture2D(u_texture, v_texcoord).r > 0.0;
    gl_FragColor = vec4(0,0,0,1);
    sum += int(texture2D(u_texture, v_texcoord + vec2( 0              , u_texel_size.y  )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size.x , u_texel_size.y  )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size.x , 0               )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( u_texel_size.x , -u_texel_size.y )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( 0              , -u_texel_size.y )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size.x, -u_texel_size.y )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size.x, 0               )).r > 0.0);
    sum += int(texture2D(u_texture, v_texcoord + vec2( -u_texel_size.x, u_texel_size.y  )).r > 0.0);
    if (alive) {
        if (sum == 2 || sum == 3) {
            gl_FragColor = vec4(1,1,1,0);
        }
    } else if (sum == 3) {
        gl_FragColor = vec4(1,1,1,0);
    }
}