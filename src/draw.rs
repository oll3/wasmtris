use core::f32::consts::PI;
use rstris::block::*;
use rstris::figure_pos::*;
use rstris::playfield::*;
use rstris::position::*;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

use nalgebra::{geometry::*, Matrix3, Matrix4, Vector2, Vector3};
use nalgebra_glm as glm;

use crate::utils::*;

struct GLBuf {
    buf_ref: web_sys::WebGlBuffer,
    js_mem_buf: js_sys::Float32Array,
    data_len: u32,
}

impl GLBuf {
    fn new(gl: &WebGlRenderingContext, data: &[f32]) -> Self {
        let mem_buf = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();

        let js_mem_buf = js_sys::Float32Array::new(&mem_buf);

        let location = data.as_ptr() as u32 / 4;
        let data_array = js_mem_buf.subarray(location, location + data.len() as u32);
        let buf_ref = gl
            .create_buffer()
            .ok_or("failed to create color buffer")
            .unwrap();
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buf_ref));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &data_array,
            WebGlRenderingContext::STATIC_DRAW,
        );

        GLBuf {
            buf_ref,
            js_mem_buf,
            data_len: data.len() as u32,
        }
    }

    fn update(&self, gl: &WebGlRenderingContext, data: &[f32]) {
        let location = data.as_ptr() as u32 / 4;
        let data_array = self
            .js_mem_buf
            .subarray(location, location + data.len() as u32);

        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buf_ref));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &data_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
    }
}

pub struct Draw {
    canvas: web_sys::HtmlCanvasElement,
    gl: WebGlRenderingContext,
    program: web_sys::WebGlProgram,

    // shader references
    projection_matrix: web_sys::WebGlUniformLocation,
    position_vertex: u32,
    vertex_color: u32,

    // tris block vertices
    block_cols: u32,
    block_rows: u32,
    blocks: Vec<f32>,
    dirty: bool,

    block_buf: GLBuf,
    color_buf: GLBuf,
}

impl Draw {
    pub fn new(canvas_id: &str, block_cols: u32, block_rows: u32) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let gl = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        let vert_shader = compile_shader(
            &gl,
            WebGlRenderingContext::VERTEX_SHADER,
            r#"
            attribute vec4 a_vertex_color;
            attribute vec2 a_position;
            uniform mat4 u_matrix;

            varying lowp vec4 vColor;

            void main() {
                gl_Position = vec4((u_matrix * vec4(a_position, 1, 1)).xy, 0, 1);
                vColor = a_vertex_color + gl_Position * 0.1;
            }
            "#,
        )
        .expect("failed to compile vertex shader");

        let frag_shader = compile_shader(
            &gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            r#"
            varying lowp vec4 vColor;
            void main() {
                gl_FragColor = vColor;
            }
            "#,
        )
        .expect("failed to compile fragment shader");
        let program =
            link_program(&gl, &vert_shader, &frag_shader).expect("failed to link shaders");

        let projection_matrix = gl
            .get_uniform_location(&program, "u_matrix")
            .expect("failed to get projection matrix");
        let position_vertex = gl.get_attrib_location(&program, "a_position") as u32;
        let vertex_color = gl.get_attrib_location(&program, "a_vertex_color") as u32;

        let block_vertex = Self::generate_blocks(&canvas, block_cols, block_rows);

        let blocks = vec![0.0; (block_cols * block_rows * 4 * 6) as usize];
        let mut vertex_colors = vec![];
        for t in 0..block_vertex.len() {
            let c = t as f32;
            let color = [0.001 * c, 0.001 * c, 1.0 - c * 0.001, 1.0];
            vertex_colors.extend_from_slice(&color);
            vertex_colors.extend_from_slice(&color);
            vertex_colors.extend_from_slice(&color);
            vertex_colors.extend_from_slice(&color);
            vertex_colors.extend_from_slice(&color);
            vertex_colors.extend_from_slice(&color);
        }

        Draw {
            block_cols,
            block_rows,
            canvas,
            block_buf: GLBuf::new(&gl, &block_vertex),
            color_buf: GLBuf::new(&gl, &blocks),
            gl,
            program,
            projection_matrix,
            position_vertex,
            vertex_color,
            dirty: true,
            blocks: blocks,
        }
    }

    fn generate_blocks(
        canvas: &web_sys::HtmlCanvasElement,
        block_cols: u32,
        block_rows: u32,
    ) -> Vec<f32> {
        let space = 1.00;
        let block_size_w = canvas.client_width() as f32 / block_cols as f32;
        let block_size_h = canvas.client_height() as f32 / block_rows as f32;
        let size = if block_size_w > block_size_h {
            block_size_h
        } else {
            block_size_w
        };

        let start_x = 0.0;
        let start_y = 0.0;

        let mut blocks = vec![];
        let mut y2 = start_y;
        for _ in 0..block_rows {
            let mut x2 = start_x;
            let bottom = y2 + space;
            let top = y2 + size - space;

            for _ in 0..block_cols {
                let left = x2 + space;
                let right = x2 + size - space;
                blocks.extend_from_slice(&[
                    left, bottom, // BL
                    right, bottom, // BR
                    left, top, // TL
                    right, bottom, // BR
                    right, top, // TR
                    left, top, // TL
                ]);
                x2 += size;
            }
            y2 += size;
        }
        blocks
    }

    fn set_projection(&self) {
        let projection_matrix: Matrix4<f32> = glm::ortho(
            0.0,
            self.canvas.client_width() as f32,
            self.canvas.client_height() as f32,
            0.0,
            0.0,
            0.0,
        );
        let mut projection_array = [0.; 16];
        projection_array.copy_from_slice(projection_matrix.as_slice());
        self.gl.uniform_matrix4fv_with_f32_array(
            Some(self.projection_matrix.as_ref()),
            false,
            &projection_array,
        );
    }

    pub fn set_block(&mut self, x: u32, y: u32, color: (f32, f32, f32, f32)) {
        if x < self.block_cols && y < self.block_rows {
            for v in 0..6 {
                let index = (6 * 4 * self.block_cols * y + x * 6 * 4 + v * 4) as usize;
                self.blocks[index + 0] = color.0;
                self.blocks[index + 1] = color.1;
                self.blocks[index + 2] = color.2;
                self.blocks[index + 3] = color.3;
            }
            self.dirty = true;
        }
    }

    pub fn draw_blocks(&mut self) {
        if self.dirty {
            self.color_buf.update(&self.gl, &self.blocks);
            self.dirty = false;
        }
        self.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        {
            // Bind the vertex buffer
            self.gl.bind_buffer(
                WebGlRenderingContext::ARRAY_BUFFER,
                Some(&self.block_buf.buf_ref),
            );
            self.gl.vertex_attrib_pointer_with_i32(
                self.position_vertex,
                2,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            self.gl.enable_vertex_attrib_array(self.position_vertex);
        }
        {
            // Bind the vertex color buffer
            self.gl.bind_buffer(
                WebGlRenderingContext::ARRAY_BUFFER,
                Some(&self.color_buf.buf_ref),
            );
            self.gl.vertex_attrib_pointer_with_i32(
                self.vertex_color,
                4,
                WebGlRenderingContext::FLOAT,
                false,
                0,
                0,
            );
            self.gl.enable_vertex_attrib_array(self.vertex_color);
        }
        self.gl.use_program(Some(&self.program));
        self.set_projection();

        self.gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (self.block_buf.data_len / 2) as i32,
        );
    }
}

fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
