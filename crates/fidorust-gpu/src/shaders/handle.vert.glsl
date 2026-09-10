#version 300 es
precision highp float;
layout(location=0) in vec2 corner;
layout(location=1) in vec2 center;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
uniform float u_radius;
out vec2 v_px;
void main() {
    vec2 screen = center * u_zoom + u_pan;
    float ext = u_radius + 1.0;
    vec2 pos = screen + corner * ext;
    vec2 clip = vec2(pos.x / u_res.x * 2.0 - 1.0, 1.0 - pos.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_px = corner * ext;
}
