precision highp float;

varying vec2 v_texcoord;

uniform sampler2D u_sand;
uniform sampler2D u_lookahead;
uniform float u_scale;
uniform vec3 u_voxel_size;
uniform vec3 u_direction;

const vec4 DARK = vec4(0.502, 0.467, 0.361, 1.0);
const vec4 LIGHT = vec4(0.796, 0.741, 0.576, 1.0);

float bilerp(in vec4 neighbors, in vec2 fuv) {
    return min(
        mix(
            mix(neighbors[0], neighbors[1], fuv.x),
            mix(neighbors[2], neighbors[3], fuv.x),
            fuv.y
        ),
    1.0);
}

void sample_neighbors(out vec4 neighbors, out vec2 fuv, in sampler2D tex, in vec2 uv, in float level) {
    vec2 st = uv / u_voxel_size.xy - 0.5;

    vec2 iuv = floor(st);
    fuv = fract(st);

    vec4 a = texture2D(tex, (iuv + vec2(0.5, 0.5)) * u_voxel_size.xy);
    vec4 b = texture2D(tex, (iuv + vec2(1.5, 0.5)) * u_voxel_size.xy);
    vec4 c = texture2D(tex, (iuv + vec2(0.5, 1.5)) * u_voxel_size.xy);
    vec4 d = texture2D(tex, (iuv + vec2(1.5, 1.5)) * u_voxel_size.xy);

    if (level == 0.0) {
        neighbors = vec4(a[0], b[0], c[0], d[0]);
    } else if (level == 1.0) {
        neighbors = vec4(a[1], b[1], c[1], d[1]);
    } else if (level == 2.0) {
        neighbors = vec4(a[2], b[2], c[2], d[2]);
    } else  {
        neighbors = vec4(a[3], b[3], c[3], d[3]);
    }
}

float max_from_lookahead(in vec2 uv, in float level) {
    vec4 neighbors;
    vec2 unused;
    sample_neighbors(neighbors, unused, u_lookahead, uv, level);
    return max(max(neighbors[0], neighbors[1]), max(neighbors[2], neighbors[3]));
}

float amount_shadowed(in vec2 uv, in float threshold) {
    vec4 neighbors;
    vec2 fuv;

    sample_neighbors(neighbors, fuv, u_sand, uv, 0.0);
    return bilerp(
        min(
            vec4(1.0, 1.0, 1.0, 1.0),
            floor(neighbors / threshold)
        ),
        fuv
    );
}


void main() {
    vec3 dir = u_voxel_size * (
        u_direction / length(u_direction.xy)
    );

    vec3 pos = vec3(v_texcoord.x, v_texcoord.y, texture2D(u_sand, v_texcoord).r);
    float level = 3.0;
    float shadowed = 0.0;
    for (int _ = 0; _ < 4096; _++) {
        pos += dir;
        if (pos.z >= 1.0) {
            break;
        }
        for (int _ = 0; _ < 4; _++) {
            if (level < 0.0) {
                break;
            }
            float height = max_from_lookahead(pos.xy, level);
            if (height < pos.z) {
                break;
            }
            level--;
        }
        if (level < 0.0) {
            shadowed = max(shadowed, amount_shadowed(pos.xy, pos.z));
            if (shadowed >= 0.99) {
                break;
            }
        } else {
            pos += dir * pow(u_scale, level + 1.0);
        }
        level = min(level + 1.0, 3.0);
    }
    gl_FragColor = LIGHT + (DARK - LIGHT) * shadowed;
}
