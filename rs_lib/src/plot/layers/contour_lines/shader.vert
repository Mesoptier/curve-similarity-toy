#version 300 es

in vec4 a_position;
out vec4 v_color;

uniform mat4 u_transform;

void main() {
    v_color = vec4(0.0, 0.0, 0.0, 0.5);
    gl_Position = a_position * u_transform;
}
