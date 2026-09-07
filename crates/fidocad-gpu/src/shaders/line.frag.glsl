#version 300 es
precision highp float;
in vec3 v_color;
in vec2 v_uv;
out vec4 frag;
void main() {
    float d = abs(v_uv.y);
    if (d > 1.0) discard;
    frag = vec4(v_color, 1.0);
}
