#version 300 es
precision highp float;
in vec2 v_px;
uniform vec3 u_color;
uniform float u_radius;
out vec4 frag;
void main() {
    float dist = length(v_px);
    float fw = max(fwidth(dist), 1e-4);
    float a = 1.0 - smoothstep(u_radius - fw, u_radius + fw, dist);
    if (a < 0.004) discard;
    frag = vec4(u_color, a);
}
