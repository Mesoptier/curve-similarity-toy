#version 300 es

in vec4 a_position;
in float a_value;
out vec4 v_color;

#define MAX_COLORS 16

uniform vec4 u_color_map[MAX_COLORS];
uniform vec2 u_value_range;

void main() {
    float min_value = u_value_range.x;
    float max_value = u_value_range.y;

    float value = (a_value - min_value) / (max_value - min_value);

    vec3 color;
    for (int idx = 1; idx < MAX_COLORS; idx++) {
        if (value < u_color_map[idx].w) {
            float w = (value - u_color_map[idx - 1].w) / (u_color_map[idx].w - u_color_map[idx - 1].w);
            color = mix(u_color_map[idx - 1].rgb, u_color_map[idx].rgb, w);
            break;
        }
    }

    v_color = vec4(color, 1.0);
    gl_Position = a_position;
}
