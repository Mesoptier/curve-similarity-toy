#version 300 es

in vec4 a_position;
in float a_value;
out vec4 v_color;

#define NUM_COLORS 256

uniform vec3 u_color_map[NUM_COLORS];
uniform vec2 u_value_range;

void main() {
    float min_value = u_value_range.x;
    float max_value = u_value_range.y;

    float value = (a_value - min_value) / (max_value - min_value);

    // TODO: Actually interpolate colors?
    int color_idx = int(value * float(NUM_COLORS - 1));
    vec3 color = u_color_map[color_idx];

    v_color = vec4(color, 1.0);
    gl_Position = a_position;
}
