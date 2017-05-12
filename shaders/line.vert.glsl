#version 150

uniform mat4 transform;
uniform mat4 model;
uniform float width;

in vec2 position;
in vec2 normal;

out vec2 f_normal;

void main() {
	f_normal = normal;
    vec4 delta = vec4(normal * width, 0, 0);
    vec4 pos = transform * model * vec4(position, 0.0, 1.0);
    gl_Position = (pos + delta);
}
