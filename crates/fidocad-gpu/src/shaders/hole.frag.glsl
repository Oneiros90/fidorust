#version 300 es
precision highp float;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
uniform vec2 u_grid;
uniform vec3 u_bg;
uniform vec3 u_gridcol;
uniform float u_show_grid;
in vec3 v_hole;
out vec4 frag;
void main() {
    vec2 screen = vec2(gl_FragCoord.x, u_res.y - gl_FragCoord.y);
    vec2 world = (screen - u_pan) / u_zoom;
    if (length(world - v_hole.xy) > v_hole.z) discard;
    if (u_show_grid < 0.5) {
        frag = vec4(u_bg, 1.0);
        return;
    }
    vec2 g = max(u_grid, vec2(1.0));
    vec2 f = abs(fract(world / g - 0.5) - 0.5);
    vec2 fw = fwidth(world / g);
    float line = 1.0 - min(smoothstep(0.0, fw.x * 1.5, f.x), smoothstep(0.0, fw.y * 1.5, f.y));
    vec2 gm = g * 5.0;
    vec2 fm = abs(fract(world / gm - 0.5) - 0.5);
    vec2 fwm = fwidth(world / gm);
    float major = 1.0 - min(smoothstep(0.0, fwm.x * 1.5, fm.x), smoothstep(0.0, fwm.y * 1.5, fm.y));
    vec3 col = mix(u_bg, u_gridcol, line * 0.35 + major * 0.25);
    frag = vec4(col, 1.0);
}
