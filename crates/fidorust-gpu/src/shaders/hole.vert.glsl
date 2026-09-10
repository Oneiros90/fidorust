#version 300 es
precision highp float;
layout(location=0) in vec2 corner;
layout(location=1) in vec3 hole;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
out vec3 v_hole;
void main() {
    float pad = 2.0 / max(u_zoom, 0.01);
    vec2 pos = hole.xy + corner * (hole.z + pad);
    vec2 screen = pos * u_zoom + u_pan;
    vec2 clip = vec2(screen.x / u_res.x * 2.0 - 1.0, 1.0 - screen.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_hole = hole;
}
