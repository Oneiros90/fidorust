#version 300 es
precision highp float;
in vec3 v_color;
in vec2 v_uv;
in vec2 v_radii;
in float v_inner;
in float v_stroke;
uniform float u_zoom;
out vec4 frag;
void main() {
    // v_uv = world_offset / radii; the ellipse is the unit circle in this space.
    // Convert implicit f=length(v_uv) to an approximate world-space signed distance
    // so stroke width stays uniform on highly eccentric ellipses.
    float d = max(length(v_uv), 1e-5);
    vec2 inv_r = 1.0 / max(v_radii, vec2(1e-4));
    vec2 grad_xy = vec2(v_uv.x * inv_r.x, v_uv.y * inv_r.y) / d;
    float grad = max(length(grad_xy), 1e-5);
    float dist = (d - 1.0) / grad;
    float fw = max(fwidth(dist), 1e-4);
    float a;
    if (v_stroke > 0.001) {
        // Opaque stroke: soft alpha tints thin curves toward the background on sRGB canvases.
        float half_w = 0.5 * (max(v_stroke, 0.175) + 0.6 / max(u_zoom, 0.01));
        if (abs(dist) > half_w) discard;
        a = 1.0;
    } else {
        float outer = 1.0 - smoothstep(-fw, fw, dist);
        float hole = 1.0;
        if (v_inner > 0.01) {
            float dist_in = (d - v_inner) / grad;
            hole = smoothstep(-fw, fw, dist_in);
        }
        a = outer * hole;
    }
    if (a < 0.004) discard;
    frag = vec4(v_color, a);
}
