#version 300 es
precision highp float;
layout(location=0) in vec2 corner;
layout(location=1) in vec2 center;
layout(location=2) in vec2 radii;
layout(location=3) in vec2 inner_stroke;
layout(location=4) in vec3 color;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
out vec3 v_color;
out vec2 v_uv;
out vec2 v_radii;
out float v_inner;
out float v_stroke;
void main() {
    float pad = max(inner_stroke.y, 0.175) * 0.5 + 4.0 / max(u_zoom, 0.01);
    vec2 ext = max(radii, vec2(0.001)) + vec2(pad);
    vec2 pos = center + corner * ext;
    vec2 screen = pos * u_zoom + u_pan;
    vec2 clip = vec2(screen.x / u_res.x * 2.0 - 1.0, 1.0 - screen.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
    v_uv = (corner * ext) / max(radii, vec2(1e-4));
    v_radii = radii;
    v_inner = inner_stroke.x;
    v_stroke = inner_stroke.y;
}
