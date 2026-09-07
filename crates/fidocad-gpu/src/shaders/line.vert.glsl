#version 300 es
precision highp float;
layout(location=0) in vec2 corner;
layout(location=1) in vec4 ab;
layout(location=2) in float width;
layout(location=3) in vec3 color;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
out vec3 v_color;
out vec2 v_uv;
out float v_len;
void main() {
    vec2 a = ab.xy;
    vec2 b = ab.zw;
    vec2 dir = b - a;
    float len = max(length(dir), 0.001);
    dir /= len;
    vec2 n = vec2(-dir.y, dir.x);
    float w = max(width, 0.175) + 0.6 / u_zoom;
    vec2 pos = mix(a, b, corner.x * 0.5 + 0.5) + n * corner.y * w * 0.5
             + dir * corner.x * w * 0.0;
    // extend caps
    pos += dir * (corner.x) * (w * 0.5);
    vec2 screen = pos * u_zoom + u_pan;
    vec2 clip = vec2(screen.x / u_res.x * 2.0 - 1.0, 1.0 - screen.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
    v_uv = corner;
    v_len = len;
}
