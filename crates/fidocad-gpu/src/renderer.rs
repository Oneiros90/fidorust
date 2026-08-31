//! WebGL2 renderer (glow). Document stays in Rust; this only uploads tessellated batches.

#![cfg(target_arch = "wasm32")]

use crate::tessellate::{CircleInstance, FillVertexGpu, LineInstance, Scene};
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
    float w = max(width, 0.35) + 1.2 / u_zoom;
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
    float a = 1.0 - smoothstep(0.85, 1.0, d);
    frag = vec4(v_color, a);
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
    float pad = max(inner_stroke.y, 0.35) * 0.5 + 4.0 / max(u_zoom, 0.01);
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
        float half_w = 0.5 * (max(v_stroke, 0.35) + 1.2 / max(u_zoom, 0.01));
        a = 1.0 - smoothstep(half_w - fw, half_w + fw, abs(dist));
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
uniform float u_grid;
uniform vec3 u_bg;
uniform vec3 u_gridcol;
out vec4 frag;
void main() {
    vec2 screen = vec2(gl_FragCoord.x, u_res.y - gl_FragCoord.y);
    vec2 world = (screen - u_pan) / u_zoom;
    float g = max(u_grid, 1.0);
    vec2 f = abs(fract(world / g - 0.5) - 0.5);
    vec2 fw = fwidth(world / g);
    float line = 1.0 - min(smoothstep(0.0, fw.x * 1.5, f.x), smoothstep(0.0, fw.y * 1.5, f.y));
    float major = 0.0;
    vec2 fm = abs(fract(world / (g * 5.0) - 0.5) - 0.5);
    vec2 fwm = fwidth(world / (g * 5.0));
    major = 1.0 - min(smoothstep(0.0, fwm.x * 1.5, fm.x), smoothstep(0.0, fwm.y * 1.5, fm.y));
    vec3 col = mix(u_bg, u_gridcol, line * 0.35 + major * 0.25);
    frag = vec4(col, 1.0);
}
"#;

fn compile(gl: &Context, vs: &str, fs: &str) -> Result<glow::Program, String> {
    unsafe {
        let program = gl.create_program().map_err(|e| e.to_string())?;
        let vs_s = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| e.to_string())?;
        gl.shader_source(vs_s, vs);
        gl.compile_shader(vs_s);
        if !gl.get_shader_compile_status(vs_s) {
            return Err(gl.get_shader_info_log(vs_s));
        }
        let fs_s = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| e.to_string())?;
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
    quad: glow::Buffer,
    line_inst: glow::Buffer,
    fill_buf: glow::Buffer,
    circ_inst: glow::Buffer,
    vao_line: glow::VertexArray,
    vao_fill: glow::VertexArray,
    vao_circ: glow::VertexArray,
    vao_grid: glow::VertexArray,
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

            let quad_data: [f32; 12] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0];
            let quad = gl.create_buffer().map_err(|e| e.to_string())?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&quad_data), glow::STATIC_DRAW);

            let line_inst = gl.create_buffer().map_err(|e| e.to_string())?;
            let fill_buf = gl.create_buffer().map_err(|e| e.to_string())?;
            let circ_inst = gl.create_buffer().map_err(|e| e.to_string())?;

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

            gl.bind_vertex_array(None);

            Ok(Self {
                gl,
                line_prog,
                fill_prog,
                circ_prog,
                grid_prog,
                quad,
                line_inst,
                fill_buf,
                circ_inst,
                vao_line,
                vao_fill,
                vao_circ,
                vao_grid,
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
        grid: f32,
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
                set1(gl, self.grid_prog, "u_grid", grid);
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
                gl.draw_arrays(glow::TRIANGLES, 0, scene.fills.len() as i32);
            }

            if !scene.lines.is_empty() {
                gl.use_program(Some(self.line_prog));
                set2(gl, self.line_prog, "u_pan", pan);
                set1(gl, self.line_prog, "u_zoom", zoom);
                set2(gl, self.line_prog, "u_res", res);
                gl.bind_vertex_array(Some(self.vao_line));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.line_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&scene.lines),
                    glow::DYNAMIC_DRAW,
                );
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, scene.lines.len() as i32);
            }

            let mut circs = scene.circles.clone();
            circs.extend_from_slice(&scene.handles);
            if !circs.is_empty() {
                gl.use_program(Some(self.circ_prog));
                set2(gl, self.circ_prog, "u_pan", pan);
                set1(gl, self.circ_prog, "u_zoom", zoom);
                set2(gl, self.circ_prog, "u_res", res);
                gl.bind_vertex_array(Some(self.vao_circ));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.circ_inst));
                gl.buffer_data_u8_slice(
                    glow::ARRAY_BUFFER,
                    bytemuck::cast_slice(&circs),
                    glow::DYNAMIC_DRAW,
                );
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, circs.len() as i32);
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

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.line_prog);
            self.gl.delete_program(self.fill_prog);
            self.gl.delete_program(self.circ_prog);
            self.gl.delete_program(self.grid_prog);
            self.gl.delete_buffer(self.quad);
            self.gl.delete_buffer(self.line_inst);
            self.gl.delete_buffer(self.fill_buf);
            self.gl.delete_buffer(self.circ_inst);
            self.gl.delete_vertex_array(self.vao_line);
            self.gl.delete_vertex_array(self.vao_fill);
            self.gl.delete_vertex_array(self.vao_circ);
            self.gl.delete_vertex_array(self.vao_grid);
        }
    }
}
