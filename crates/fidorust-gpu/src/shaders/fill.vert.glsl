#version 300 es
precision highp float;
layout(location=0) in vec2 pos;
layout(location=1) in vec4 color;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
out vec4 v_color;
void main() {
    vec2 screen = pos * u_zoom + u_pan;
    vec2 clip = vec2(screen.x / u_res.x * 2.0 - 1.0, 1.0 - screen.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
}
