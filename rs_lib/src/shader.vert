#version 300 es

in vec4 a_position;
in float a_value;
out vec4 v_color;

#define MAX_COLORS 16

uniform vec4 u_color_map[MAX_COLORS];
uniform vec2 u_value_range;

vec3 color_map_get(float value) {
    // TODO: Pass u_color_map_len as uniform along with u_color_map
    int u_color_map_len = 12;
    for (int idx = 1; idx < u_color_map_len; idx++) {
        if (value < u_color_map[idx].w) {
            float w = (value - u_color_map[idx - 1].w) / (u_color_map[idx].w - u_color_map[idx - 1].w);
            return mix(u_color_map[idx - 1].rgb, u_color_map[idx].rgb, w);
        }
    }
    return u_color_map[u_color_map_len - 1].rgb;
}

void main() {
    if (a_value < 0.0) {
        v_color = vec4(0.0, 0.0, 0.0, 0.5);
    } else {
        float min_value = u_value_range.x;
        float max_value = u_value_range.y;
        float value = (a_value - min_value) / (max_value - min_value);
        vec3 color = color_map_get(value);
        v_color = vec4(color, 1.0);
    }

    gl_Position = a_position;
}
