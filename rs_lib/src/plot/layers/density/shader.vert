#version 300 es

in vec4 a_position;
in float a_value;

out float v_gradient_coord;

uniform vec2 u_value_range;
uniform mat4 u_transform;

void main() {
    float min_value = u_value_range.x;
    float max_value = u_value_range.y;
    v_gradient_coord = (a_value - min_value) / (max_value - min_value);
    gl_Position = a_position * u_transform;
}
