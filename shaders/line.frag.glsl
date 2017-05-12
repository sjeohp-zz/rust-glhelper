#version 150

uniform float width;

in vec2 f_normal;

out vec4 out_color;

void main() {

	float feather = 0.0;

	float l = length(f_normal) * width;

	float d = 1 - l;
	float a;

	if (l > width - feather) {
		d = l - (width - feather);
		a = 1 - d / feather;
	} else {
		a = 1;
	}

	out_color = vec4(0, 0, 0, a);
}
