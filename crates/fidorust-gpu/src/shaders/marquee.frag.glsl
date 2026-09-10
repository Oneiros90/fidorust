#version 300 es
precision highp float;
uniform vec4 u_rect;
uniform vec2 u_res;
uniform vec3 u_color;
out vec4 frag;
void main() {
    // Integer canvas pixel (top-left origin), 2 px border.
    ivec2 ip = ivec2(floor(vec2(gl_FragCoord.x, u_res.y - gl_FragCoord.y)));
    int x0 = int(round(min(u_rect.x, u_rect.z)));
    int y0 = int(round(min(u_rect.y, u_rect.w)));
    int x1 = int(round(max(u_rect.x, u_rect.z)));
    int y1 = int(round(max(u_rect.y, u_rect.w)));
    if (x0 == x1 && y0 == y1) discard;

    const int border = 2;
    bool in_x = ip.x >= x0 && ip.x <= x1;
    bool in_y = ip.y >= y0 && ip.y <= y1;
    bool on_left = ip.x >= x0 && ip.x < x0 + border && in_y;
    bool on_right = ip.x <= x1 && ip.x > x1 - border && in_y;
    bool on_top = ip.y >= y0 && ip.y < y0 + border && in_x;
    bool on_bottom = ip.y <= y1 && ip.y > y1 - border && in_x;
    if (!(on_left || on_right || on_top || on_bottom)) discard;

    // Each edge dashes from the top-left anchor: top/bottom left→right, left/right top→down.
    float t;
    if (on_top) {
        t = float(ip.x - x0);
    } else if (on_left) {
        t = float(ip.y - y0);
    } else if (on_right) {
        t = float(ip.y - y0);
    } else {
        t = float(ip.x - x0);
    }

    const float dash = 6.0;
    const float gap = 4.0;
    if (mod(t, dash + gap) >= dash) discard;

    frag = vec4(u_color, 1.0);
}
