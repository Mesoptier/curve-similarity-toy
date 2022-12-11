use std::{cell::{Cell, RefCell}, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

#[wasm_bindgen]
pub fn start(
    context: &WebGl2RenderingContext,
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    start_for_real(context, width, height)
}

pub fn start_for_real(
    context: &WebGl2RenderingContext,
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    let context = context.clone();

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es

        in vec4 a_position;
        in vec4 a_color;
        out vec4 v_color;

        void main() {
            v_color = a_color;
            gl_Position = a_position;
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es

        precision highp float;

        in vec4 v_color;
        out vec4 out_color;

        void main() {
            out_color = v_color;
        }
        "##,
    )?;

    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    /// The function to plot
    fn f(x: f32, y: f32, t: f32) -> f32 {
        ((x * y * 10.0 + t).sin() + 1.0) / 2.0
    }

    /// Maps from t in [0, 1] to the color that should be rendered
    fn c(t: f32) -> [f32; 3] {
        // TODO: This exposes a path on the local filesystem in the compiled WASM
        let gradient = palette::gradient::named::PLASMA;
        let color = gradient.get(t);
        [color.red, color.green, color.blue]
    }

    const BYTES_PER_FLOAT: i32 = 4;

    const FLOATS_PER_POSITION: i32 = 2;
    const FLOATS_PER_COLOR: i32 = 3;
    const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_COLOR;

    let x_len = 100;
    let y_len = 100;

    fn build_vertex_data(x_len: i32, y_len: i32, t: f32) -> Vec<f32> {
        let vertex_data_len = ((x_len * y_len) * FLOATS_PER_VERTEX) as usize;
        let mut vertex_data: Vec<f32> = Vec::with_capacity(vertex_data_len);

        for y_idx in 0..y_len {
            for x_idx in 0..x_len {
                let x: f32 = (x_idx as f32 / (x_len - 1) as f32) * 2.0 - 1.0;
                let y: f32 = (y_idx as f32 / (y_len - 1) as f32) * 2.0 - 1.0;
                vertex_data.extend([x, y]); // FLOATS_PER_POSITION
                vertex_data.extend(c(f(x, y, t))); // FLOATS_PER_COLOR
            }
        }

        vertex_data
    }

    fn build_index_data(x_len: i32, y_len: i32) -> Vec<u16> {
        let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;
        let mut index_data: Vec<u16> = Vec::with_capacity(index_data_len);

        for y in 0..(y_len - 1) {
            if y > 0 {
                // Degenerate begin: repeat first vertex
                index_data.push((y * x_len) as u16);
            }

            for x in 0..x_len {
                index_data.push(((y * x_len) + x) as u16);
                index_data.push((((y + 1) * x_len) + x) as u16);
            }

            if y < y_len - 2 {
                // Degenerate end: repeat last vertex
                index_data.push((((y + 1) * x_len) + (x_len - 1)) as u16);
            }
        }

        index_data
    }

    // Buffers that will hold vertex and index data
    let vertex_buffer =
        context.create_buffer().ok_or("Failed to create buffer")?;
    let index_buffer =
        context.create_buffer().ok_or("Failed to create buffer")?;

    // VertexArrayObject used for drawing the main triangle strip
    let vao_triangles = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao_triangles));

    context.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&vertex_buffer),
    );
    context.bind_buffer(
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&index_buffer),
    );

    // VertexArrayObject used for drawing the mesh of the main triangle strip
    let vao_lines = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao_lines));

    context.bind_buffer(
        WebGl2RenderingContext::ARRAY_BUFFER,
        Some(&vertex_buffer),
    );
    context.bind_buffer(
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&index_buffer),
    );

    // Build and upload index data
    let index_data = build_index_data(x_len, y_len);
    unsafe {
        let index_data_array_buf_view = js_sys::Uint16Array::view(&index_data);
        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_data_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        )
    }

    // Bind attributes
    let position_attribute =
        context.get_attrib_location(&program, "a_position") as u32;
    let color_attribute =
        context.get_attrib_location(&program, "a_color") as u32;

    // - Main triangles
    context.bind_vertex_array(Some(&vao_triangles));

    context.vertex_attrib_pointer_with_i32(
        position_attribute,
        FLOATS_PER_POSITION,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute);

    context.vertex_attrib_pointer_with_i32(
        color_attribute,
        FLOATS_PER_COLOR,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        FLOATS_PER_POSITION * BYTES_PER_FLOAT,
    );
    context.enable_vertex_attrib_array(color_attribute);

    // - Mesh of main triangles (note: constant color)
    context.bind_vertex_array(Some(&vao_lines));

    context.vertex_attrib_pointer_with_i32(
        position_attribute,
        FLOATS_PER_POSITION,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute);

    context.vertex_attrib3f(color_attribute, 0.0, 0.0, 0.0);

    // UI
    let show_mesh = Rc::new(Cell::new(false));
    let is_playing = Rc::new(Cell::new(true));

    // Event listeners
    let document = web_sys::window().unwrap().document().unwrap();

    {
        let show_mesh = show_mesh.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            show_mesh.set(!show_mesh.get());
        });
        let button = document.get_element_by_id("toggle-show-mesh").unwrap();
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let is_playing = is_playing.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            is_playing.set(!is_playing.get());
        });
        let button = document.get_element_by_id("toggle-playing").unwrap();
        button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Begin draw loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut frame = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if is_playing.get() {
            frame += 1;
        }

        // Update vertex data
        let vertex_data =
            build_vertex_data(x_len, y_len, (frame as f32) / 60.0);

        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&vertex_buffer),
        );

        unsafe {
            let vertex_data_array_buf_view =
                js_sys::Float32Array::view(&vertex_data);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vertex_data_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            )
        }

        // Draw
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        context.bind_vertex_array(Some(&vao_triangles));
        context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            index_data.len() as i32,
            WebGl2RenderingContext::UNSIGNED_SHORT,
            0,
        );

        if show_mesh.get() {
            context.bind_vertex_array(Some(&vao_lines));
            context.draw_elements_with_i32(
                WebGl2RenderingContext::LINE_STRIP,
                index_data.len() as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );
        }

        context.bind_vertex_array(None);

        let finished = false;
        if finished {
            // Drop our handle to this closure so that it will get cleaned up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| "Unable to create shader object".to_string())?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| "Unknown error compiling shader".to_string()))
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| "Unable to create program object".to_string())?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| "Unknown error linking program".to_string()))
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
