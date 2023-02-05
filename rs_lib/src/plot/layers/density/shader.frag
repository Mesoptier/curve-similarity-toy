#version 300 es

precision highp float;

in float v_gradient_coord;

out vec4 out_color;

uniform sampler2D u_gradient;

void main() {
    out_color = texture(u_gradient, vec2(v_gradient_coord, 0.5));
}
