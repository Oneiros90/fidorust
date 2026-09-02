//! WebGL2 renderer (glow). Document stays in Rust; this only uploads tessellated batches.

#![cfg(target_arch = "wasm32")]

use crate::tessellate::{CircleInstance, FillVertexGpu, LineInstance, PadHole, Scene};
use fidocad_core::layers::LAYER_COUNT;
use glow::{Context, HasContext};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

const LINE_VS: &str = r#"#version 300 es
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
"#;

const LINE_FS: &str = r#"#version 300 es
precision highp float;
in vec3 v_color;
in vec2 v_uv;
out vec4 frag;
void main() {
    float d = abs(v_uv.y);
    if (d > 1.0) discard;
    frag = vec4(v_color, 1.0);
}
"#;

const FILL_VS: &str = r#"#version 300 es
precision highp float;
layout(location=0) in vec2 pos;
layout(location=1) in vec3 color;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
out vec3 v_color;
void main() {
    vec2 screen = pos * u_zoom + u_pan;
    vec2 clip = vec2(screen.x / u_res.x * 2.0 - 1.0, 1.0 - screen.y / u_res.y * 2.0);
    gl_Position = vec4(clip, 0.0, 1.0);
    v_color = color;
}
"#;

const FILL_FS: &str = r#"#version 300 es
precision highp float;
in vec3 v_color;
out vec4 frag;
void main() { frag = vec4(v_color, 1.0); }
"#;

const CIRC_VS: &str = r#"#version 300 es
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
"#;

const CIRC_FS: &str = r#"#version 300 es
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
"#;

const HOLE_VS: &str = r#"#version 300 es
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
"#;

const HOLE_FS: &str = r#"#version 300 es
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
"#;

const GRID_VS: &str = r#"#version 300 es
precision highp float;
layout(location=0) in vec2 pos;
void main() { gl_Position = vec4(pos, 0.0, 1.0); }
"#;

const GRID_FS: &str = r#"#version 300 es
precision highp float;
uniform vec2 u_pan;
uniform float u_zoom;
uniform vec2 u_res;
uniform vec2 u_grid;
uniform vec3 u_bg;
uniform vec3 u_gridcol;
out vec4 frag;
void main() {
    vec2 screen = vec2(gl_FragCoord.x, u_res.y - gl_FragCoord.y);
    vec2 world = (screen - u_pan) / u_zoom;
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
"#;

const MARQUEE_FS: &str = r#"#version 300 es
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
"#;

fn compile(gl: &Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
    unsafe {
        let program = gl.create_program().map_err(|e| e.to_string())?;
        let vs_s = gl
            .create_shader(glow::VERTEX_SHADER)
            .map_err(|e| e.to_string())?;
        gl.shader_source(vs_s, vs);
        gl.compile_shader(vs_s);
        if !gl.get_shader_compile_status(vs_s) {
            return Err(gl.get_shader_info_log(vs_s));
        }
        let fs_s = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .map_err(|e| e.to_string())?;
        gl.shader_source(fs_s, fs);
        gl.compile_shader(fs_s);
        if !gl.get_shader_compile_status(fs_s) {
            return Err(gl.get_shader_info_log(fs_s));
        }
        gl.attach_shader(program, vs_s);
        gl.attach_shader(program, fs_s);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            return Err(gl.get_program_info_log(program));
        }
        gl.delete_shader(vs_s);
        gl.delete_shader(fs_s);
        Ok(program)
    }
}

pub struct Renderer {
    gl: Context,
    line_prog: glow::Program,
    fill_prog: glow::Program,
    circ_prog: glow::Program,
    grid_prog: glow::Program,
    hole_prog: glow::Program,
    marquee_prog: glow::Program,
    quad: glow::Buffer,
    line_inst: glow::Buffer,
    fill_buf: glow::Buffer,
    circ_inst: glow::Buffer,
    hole_inst: glow::Buffer,
    vao_line: glow::VertexArray,
    vao_fill: glow::VertexArray,
    vao_circ: glow::VertexArray,
    vao_grid: glow::VertexArray,
    vao_hole: glow::VertexArray,
    vao_marquee: glow::VertexArray,
    bg: [f32; 3],
    grid: [f32; 3],
}

impl Renderer {
    pub fn from_canvas(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let gl2 = canvas
            .get_context("webgl2")
            .map_err(|e| format!("{e:?}"))?
            .ok_or("WebGL2 not available")?
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| "not webgl2")?;
        let gl = Context::from_webgl2_context(gl2);
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            let line_prog = compile(&gl, LINE_VS, LINE_FS)?;
            let fill_prog = compile(&gl, FILL_VS, FILL_FS)?;
            let circ_prog = compile(&gl, CIRC_VS, CIRC_FS)?;
            let grid_prog = compile(&gl, GRID_VS, GRID_FS)?;
            let hole_prog = compile(&gl, HOLE_VS, HOLE_FS)?;
            let marquee_prog = compile(&gl, GRID_VS, MARQUEE_FS)?;

            let quad_data: [f32; 12] = [
                -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0,
            ];
            let quad = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&quad_data),
                glow::STATIC_DRAW,
            );

            let line_inst = gl.create_buffer().map_err(|e| e.to_string())?;
            let fill_buf = gl.create_buffer().map_err(|e| e.to_string())?;
            let circ_inst = gl.create_buffer().map_err(|e| e.to_string())?;
            let hole_inst = gl.create_buffer().map_err(|e| e.to_string())?;

            let vao_line = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_line));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(line_inst));
            let stride = std::mem::size_of::<LineInstance>() as i32;
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, 0);
            gl.vertex_attrib_divisor(1, 1);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, stride, 16);
            gl.vertex_attrib_divisor(2, 1);
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(3, 3, glow::FLOAT, false, stride, 20);
            gl.vertex_attrib_divisor(3, 1);

            let vao_fill = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_fill));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(fill_buf));
            let fs = std::mem::size_of::<FillVertexGpu>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, fs, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, fs, 8);

            let vao_circ = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_circ));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(circ_inst));
            let cs = std::mem::size_of::<CircleInstance>() as i32;
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, cs, 0);
            gl.vertex_attrib_divisor(1, 1);
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, cs, 8);
            gl.vertex_attrib_divisor(2, 1);
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(3, 2, glow::FLOAT, false, cs, 16);
            gl.vertex_attrib_divisor(3, 1);
            gl.enable_vertex_attrib_array(4);
            gl.vertex_attrib_pointer_f32(4, 3, glow::FLOAT, false, cs, 24);
            gl.vertex_attrib_divisor(4, 1);

            let vao_grid = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_grid));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);

            let vao_hole = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_hole));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(hole_inst));
            let hs = std::mem::size_of::<PadHole>() as i32;
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, hs, 0);
            gl.vertex_attrib_divisor(1, 1);

            let vao_marquee = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_marquee));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);

            gl.bind_vertex_array(None);

            Ok(Self {
                gl,
                line_prog,
                fill_prog,
                circ_prog,
                grid_prog,
                hole_prog,
                marquee_prog,
                quad,
                line_inst,
                fill_buf,
                circ_inst,
                hole_inst,
                vao_line,
                vao_fill,
                vao_circ,
                vao_grid,
                vao_hole,
                vao_marquee,
                bg: [0.97, 0.95, 0.92],
                grid: [0.78, 0.72, 0.66],
            })
        }
    }

    pub fn set_theme(&mut self, bg: [f32; 3], grid: [f32; 3]) {
        self.bg = bg;
        self.grid = grid;
    }

    pub fn draw(
        &mut self,
        scene: &Scene,
        pan: (f32, f32),
        zoom: f32,
        res: (f32, f32),
        grid: (f32, f32),
        show_grid: bool,
    ) {
        unsafe {
            let gl = &self.gl;
            gl.viewport(0, 0, res.0 as i32, res.1 as i32);
            gl.clear_color(self.bg[0], self.bg[1], self.bg[2], 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            if show_grid {
                gl.use_program(Some(self.grid_prog));
                set2(gl, self.grid_prog, "u_pan", pan);
                set1(gl, self.grid_prog, "u_zoom", zoom);
                set2(gl, self.grid_prog, "u_res", res);
                set2(gl, self.grid_prog, "u_grid", grid);
                set3(gl, self.grid_prog, "u_bg", self.bg);
                set3(gl, self.grid_prog, "u_gridcol", self.grid);
                gl.bind_vertex_array(Some(self.vao_grid));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
            }

            if !scene.fills.is_empty() {
                gl.use_program(Some(self.fill_prog));
                set2(gl, self.fill_prog, "u_pan", pan);
                set1(gl, self.fill_prog, "u_zoom", zoom);
                set2(gl, self.fill_prog, "u_res", res);
                gl.bind_vertex_array(Some(self.vao_fill));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.fill_buf));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.fills),
                    glow::DYNAMIC_DRAW,
                );
            }
            if !scene.lines.is_empty() {
                gl.bind_vertex_array(Some(self.vao_line));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.line_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.lines),
                    glow::DYNAMIC_DRAW,
                );
            }
            if !scene.circles.is_empty() {
                gl.bind_vertex_array(Some(self.vao_circ));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.circ_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.circles),
                    glow::DYNAMIC_DRAW,
                );
            }
            if !scene.pad_holes.is_empty() {
                gl.bind_vertex_array(Some(self.vao_hole));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.hole_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.pad_holes),
                    glow::DYNAMIC_DRAW,
                );
            }

            for i in 0..LAYER_COUNT {
                let (f0, fn_) = layer_span(i, &scene.layer_fill_end);
                if fn_ > 0 {
                    gl.use_program(Some(self.fill_prog));
                    gl.bind_vertex_array(Some(self.vao_fill));
                    gl.draw_arrays(glow::TRIANGLES, f0, fn_);
                }
                let (l0, ln) = layer_span(i, &scene.layer_line_end);
                if ln > 0 {
                    gl.use_program(Some(self.line_prog));
                    set2(gl, self.line_prog, "u_pan", pan);
                    set1(gl, self.line_prog, "u_zoom", zoom);
                    set2(gl, self.line_prog, "u_res", res);
                    bind_line_instances(gl, self.vao_line, self.line_inst, l0);
                    gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, ln);
                }
                let (c0, cn) = layer_span(i, &scene.layer_circ_end);
                if cn > 0 {
                    gl.use_program(Some(self.circ_prog));
                    set2(gl, self.circ_prog, "u_pan", pan);
                    set1(gl, self.circ_prog, "u_zoom", zoom);
                    set2(gl, self.circ_prog, "u_res", res);
                    bind_circle_instances(gl, self.vao_circ, self.circ_inst, c0);
                    gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, cn);
                }
                let (h0, hn) = layer_span(i, &scene.layer_hole_end);
                if hn > 0 {
                    gl.use_program(Some(self.hole_prog));
                    set2(gl, self.hole_prog, "u_pan", pan);
                    set1(gl, self.hole_prog, "u_zoom", zoom);
                    set2(gl, self.hole_prog, "u_res", res);
                    set2(gl, self.hole_prog, "u_grid", grid);
                    set3(gl, self.hole_prog, "u_bg", self.bg);
                    set3(gl, self.hole_prog, "u_gridcol", self.grid);
                    set1(
                        gl,
                        self.hole_prog,
                        "u_show_grid",
                        if show_grid { 1.0 } else { 0.0 },
                    );
                    bind_hole_instances(gl, self.vao_hole, self.hole_inst, h0);
                    gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, hn);
                }
            }

            if !scene.handles.is_empty() {
                gl.use_program(Some(self.circ_prog));
                set2(gl, self.circ_prog, "u_pan", pan);
                set1(gl, self.circ_prog, "u_zoom", zoom);
                set2(gl, self.circ_prog, "u_res", res);
                bind_circle_instances(gl, self.vao_circ, self.circ_inst, 0);
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.circ_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.handles),
                    glow::DYNAMIC_DRAW,
                );
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, scene.handles.len() as i32);
            }

            if let Some(rect) = scene.marquee {
                gl.use_program(Some(self.marquee_prog));
                set4(gl, self.marquee_prog, "u_rect", rect);
                set2(gl, self.marquee_prog, "u_res", res);
                set3(gl, self.marquee_prog, "u_color", scene.marquee_color);
                gl.bind_vertex_array(Some(self.vao_marquee));
                gl.draw_arrays(glow::TRIANGLES, 0, 6);
            }

            gl.bind_vertex_array(None);
        }
    }
}

unsafe fn set1(gl: &Context, prog: glow::Program, name: &str, v: f32) {
    let loc = gl.get_uniform_location(prog, name);
    gl.uniform_1_f32(loc.as_ref(), v);
}
unsafe fn set2(gl: &Context, prog: glow::Program, name: &str, v: (f32, f32)) {
    let loc = gl.get_uniform_location(prog, name);
    gl.uniform_2_f32(loc.as_ref(), v.0, v.1);
}
unsafe fn set3(gl: &Context, prog: glow::Program, name: &str, v: [f32; 3]) {
    let loc = gl.get_uniform_location(prog, name);
    gl.uniform_3_f32(loc.as_ref(), v[0], v[1], v[2]);
}
unsafe fn set4(gl: &Context, prog: glow::Program, name: &str, v: [f32; 4]) {
    let loc = gl.get_uniform_location(prog, name);
    gl.uniform_4_f32(loc.as_ref(), v[0], v[1], v[2], v[3]);
}

fn layer_span(i: usize, ends: &[u32; LAYER_COUNT]) -> (i32, i32) {
    let start = if i == 0 { 0 } else { ends[i - 1] };
    (start as i32, (ends[i] - start) as i32)
}

unsafe fn bind_line_instances(gl: &Context, vao: glow::VertexArray, buf: glow::Buffer, first: i32) {
    let stride = std::mem::size_of::<LineInstance>() as i32;
    let off = first as i32 * stride;
    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
    gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, stride, off);
    gl.vertex_attrib_pointer_f32(2, 1, glow::FLOAT, false, stride, off + 16);
    gl.vertex_attrib_pointer_f32(3, 3, glow::FLOAT, false, stride, off + 20);
}

unsafe fn bind_circle_instances(gl: &Context, vao: glow::VertexArray, buf: glow::Buffer, first: i32) {
    let stride = std::mem::size_of::<CircleInstance>() as i32;
    let off = first as i32 * stride;
    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
    gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, off);
    gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, off + 8);
    gl.vertex_attrib_pointer_f32(3, 2, glow::FLOAT, false, stride, off + 16);
    gl.vertex_attrib_pointer_f32(4, 3, glow::FLOAT, false, stride, off + 24);
}

unsafe fn bind_hole_instances(gl: &Context, vao: glow::VertexArray, buf: glow::Buffer, first: i32) {
    let stride = std::mem::size_of::<PadHole>() as i32;
    let off = first as i32 * stride;
    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
    gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, off);
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.line_prog);
            self.gl.delete_program(self.fill_prog);
            self.gl.delete_program(self.circ_prog);
            self.gl.delete_program(self.grid_prog);
            self.gl.delete_program(self.hole_prog);
            self.gl.delete_program(self.marquee_prog);
            self.gl.delete_buffer(self.quad);
            self.gl.delete_buffer(self.line_inst);
            self.gl.delete_buffer(self.fill_buf);
            self.gl.delete_buffer(self.circ_inst);
            self.gl.delete_buffer(self.hole_inst);
            self.gl.delete_vertex_array(self.vao_line);
            self.gl.delete_vertex_array(self.vao_fill);
            self.gl.delete_vertex_array(self.vao_circ);
            self.gl.delete_vertex_array(self.vao_grid);
            self.gl.delete_vertex_array(self.vao_hole);
            self.gl.delete_vertex_array(self.vao_marquee);
        }
    }
}
