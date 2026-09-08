//! WebGL2 renderer (glow). Document stays in Rust; this only uploads tessellated batches.

#![cfg(target_arch = "wasm32")]

use crate::scene::{CircleInstance, FillVertexGpu, LineInstance, PadHole, Scene};
use crate::theme::Theme;
use glow::{Context, HasContext};
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

const LINE_VS: &str = include_str!("shaders/line.vert.glsl");
const LINE_FS: &str = include_str!("shaders/line.frag.glsl");
const FILL_VS: &str = include_str!("shaders/fill.vert.glsl");
const FILL_FS: &str = include_str!("shaders/fill.frag.glsl");
const CIRC_VS: &str = include_str!("shaders/circle.vert.glsl");
const CIRC_FS: &str = include_str!("shaders/circle.frag.glsl");
const HOLE_VS: &str = include_str!("shaders/hole.vert.glsl");
const HOLE_FS: &str = include_str!("shaders/hole.frag.glsl");
const GRID_VS: &str = include_str!("shaders/grid.vert.glsl");
const GRID_FS: &str = include_str!("shaders/grid.frag.glsl");
const MARQUEE_FS: &str = include_str!("shaders/marquee.frag.glsl");

/// `(location, component count, byte offset)` within one instance (or vertex).
type AttribLayout = [(u32, i32, i32)];

const LINE_LAYOUT: &AttribLayout = &[(1, 4, 0), (2, 1, 16), (3, 4, 20)];
const FILL_LAYOUT: &AttribLayout = &[(0, 2, 0), (1, 4, 8)];
const CIRC_LAYOUT: &AttribLayout = &[(1, 2, 0), (2, 2, 8), (3, 2, 16), (4, 4, 24)];
const HOLE_LAYOUT: &AttribLayout = &[(1, 3, 0)];

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
            bind_quad_corners(&gl, quad);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(line_inst));
            setup_instanced(&gl, LINE_LAYOUT, std::mem::size_of::<LineInstance>() as i32);

            let vao_fill = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_fill));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(fill_buf));
            setup_attribs(
                &gl,
                FILL_LAYOUT,
                std::mem::size_of::<FillVertexGpu>() as i32,
                0,
                false,
            );

            let vao_circ = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_circ));
            bind_quad_corners(&gl, quad);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(circ_inst));
            setup_instanced(
                &gl,
                CIRC_LAYOUT,
                std::mem::size_of::<CircleInstance>() as i32,
            );

            let vao_grid = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_grid));
            bind_quad_corners(&gl, quad);

            let vao_hole = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_hole));
            bind_quad_corners(&gl, quad);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(hole_inst));
            setup_instanced(&gl, HOLE_LAYOUT, std::mem::size_of::<PadHole>() as i32);

            let vao_marquee = gl.create_vertex_array().map_err(|e| e.to_string())?;
            gl.bind_vertex_array(Some(vao_marquee));
            bind_quad_corners(&gl, quad);

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
                bg: Theme::LIGHT.bg,
                grid: Theme::LIGHT.grid,
            })
        }
    }

    pub fn set_theme(&mut self, bg: [f32; 3], grid: [f32; 3]) {
        self.bg = bg;
        self.grid = grid;
    }

    pub fn set_theme_enum(&mut self, theme: &Theme) {
        self.set_theme(theme.bg, theme.grid);
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
        }
        if show_grid {
            self.draw_grid(pan, zoom, res, grid);
        }
        self.upload(scene, pan, zoom, res);
        for i in 0..scene.layer_fill_end.len() {
            self.draw_layer(i, scene, pan, zoom, res, grid, show_grid);
        }
        self.draw_handles(scene, pan, zoom, res);
        self.draw_marquee(scene, res);
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }

    fn upload(&self, scene: &Scene, pan: (f32, f32), zoom: f32, res: (f32, f32)) {
        unsafe {
            let gl = &self.gl;
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
        }
    }

    fn draw_grid(&self, pan: (f32, f32), zoom: f32, res: (f32, f32), grid: (f32, f32)) {
        unsafe {
            let gl = &self.gl;
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
    }

    fn draw_layer(
        &self,
        i: usize,
        scene: &Scene,
        pan: (f32, f32),
        zoom: f32,
        res: (f32, f32),
        grid: (f32, f32),
        show_grid: bool,
    ) {
        unsafe {
            let gl = &self.gl;
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
                bind_instances(
                    gl,
                    self.vao_line,
                    self.line_inst,
                    l0,
                    std::mem::size_of::<LineInstance>() as i32,
                    LINE_LAYOUT,
                );
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, ln);
            }
            let (c0, cn) = layer_span(i, &scene.layer_circ_end);
            if cn > 0 {
                gl.use_program(Some(self.circ_prog));
                set2(gl, self.circ_prog, "u_pan", pan);
                set1(gl, self.circ_prog, "u_zoom", zoom);
                set2(gl, self.circ_prog, "u_res", res);
                bind_instances(
                    gl,
                    self.vao_circ,
                    self.circ_inst,
                    c0,
                    std::mem::size_of::<CircleInstance>() as i32,
                    CIRC_LAYOUT,
                );
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
                bind_instances(
                    gl,
                    self.vao_hole,
                    self.hole_inst,
                    h0,
                    std::mem::size_of::<PadHole>() as i32,
                    HOLE_LAYOUT,
                );
                gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, hn);
            }
        }
    }

    fn draw_handles(&self, scene: &Scene, pan: (f32, f32), zoom: f32, res: (f32, f32)) {
        if scene.handles.is_empty() {
            return;
        }
        unsafe {
            let gl = &self.gl;
            gl.use_program(Some(self.circ_prog));
            set2(gl, self.circ_prog, "u_pan", pan);
            set1(gl, self.circ_prog, "u_zoom", zoom);
            set2(gl, self.circ_prog, "u_res", res);
            bind_instances(
                gl,
                self.vao_circ,
                self.circ_inst,
                0,
                std::mem::size_of::<CircleInstance>() as i32,
                CIRC_LAYOUT,
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.circ_inst));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&scene.handles),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, 6, scene.handles.len() as i32);
        }
    }

    fn draw_marquee(&self, scene: &Scene, res: (f32, f32)) {
        let Some(rect) = scene.marquee else {
            return;
        };
        unsafe {
            let gl = &self.gl;
            gl.use_program(Some(self.marquee_prog));
            set4(gl, self.marquee_prog, "u_rect", rect);
            set2(gl, self.marquee_prog, "u_res", res);
            set3(gl, self.marquee_prog, "u_color", scene.marquee_color);
            gl.bind_vertex_array(Some(self.vao_marquee));
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

unsafe fn bind_quad_corners(gl: &Context, quad: glow::Buffer) {
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(quad));
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
}

unsafe fn setup_attribs(
    gl: &Context,
    layout: &AttribLayout,
    stride: i32,
    base: i32,
    instanced: bool,
) {
    for &(location, size, offset) in layout {
        gl.enable_vertex_attrib_array(location);
        gl.vertex_attrib_pointer_f32(location, size, glow::FLOAT, false, stride, base + offset);
        if instanced {
            gl.vertex_attrib_divisor(location, 1);
        }
    }
}

unsafe fn setup_instanced(gl: &Context, layout: &AttribLayout, stride: i32) {
    setup_attribs(gl, layout, stride, 0, true);
}

unsafe fn bind_instances(
    gl: &Context,
    vao: glow::VertexArray,
    buf: glow::Buffer,
    first: i32,
    stride: i32,
    layout: &AttribLayout,
) {
    let off = first * stride;
    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf));
    for &(location, size, offset) in layout {
        gl.vertex_attrib_pointer_f32(location, size, glow::FLOAT, false, stride, off + offset);
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

fn layer_span(i: usize, ends: &[u32]) -> (i32, i32) {
    let start = if i == 0 { 0 } else { ends[i - 1] };
    (start as i32, (ends[i] - start) as i32)
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
