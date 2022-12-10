use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

#[wasm_bindgen]
pub fn start(
    context: &WebGl2RenderingContext,
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
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

    const BYTES_PER_FLOAT: i32 = 4;

    const FLOATS_PER_POSITION: i32 = 2;
    const FLOATS_PER_COLOR: i32 = 3;
    const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_COLOR;

    fn f(x: f32, y: f32) -> f32 {
        (x.powi(2) + y.powi(2)) / 2.0
    }

    // TODO: Render using triangle strips instead
    let mut vertices = vec![];

    const STEPS: usize = 100;
    const CELL_SIZE: f32 = 2.0 / (STEPS as f32);

    for x_step in 0..STEPS {
        let x: f32 = (x_step as f32 / STEPS as f32) * 2.0 - 1.0;

        for y_step in 0..STEPS {
            let y: f32 = (y_step as f32 / STEPS as f32) * 2.0 - 1.0;

            vertices.extend([x, y]);
            vertices.extend([f(x, y), 0.0, 0.0]);
            vertices.extend([x + CELL_SIZE, y]);
            vertices.extend([f(x + CELL_SIZE, y), 0.0, 0.0]);
            vertices.extend([x, y + CELL_SIZE]);
            vertices.extend([f(x, y + CELL_SIZE), 0.0, 0.0]);

            vertices.extend([x + CELL_SIZE, y + CELL_SIZE]);
            vertices.extend([f(x + CELL_SIZE, y + CELL_SIZE), 0.0, 0.0]);
            vertices.extend([x + CELL_SIZE, y]);
            vertices.extend([f(x + CELL_SIZE, y), 0.0, 0.0]);
            vertices.extend([x, y + CELL_SIZE]);
            vertices.extend([f(x, y + CELL_SIZE), 0.0, 0.0]);
        }
    }

    let position_attribute =
        context.get_attrib_location(&program, "a_position");
    let color_attribute = context.get_attrib_location(&program, "a_color");

    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    unsafe {
        let vertices_array_buf_view = js_sys::Float32Array::view(&vertices);
        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vertices_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        )
    }

    let vbo = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vbo));

    // Bind attributes
    context.vertex_attrib_pointer_with_i32(
        position_attribute as u32,
        FLOATS_PER_POSITION,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute as u32);

    context.vertex_attrib_pointer_with_i32(
        color_attribute as u32,
        FLOATS_PER_COLOR,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        FLOATS_PER_POSITION * BYTES_PER_FLOAT,
    );
    context.enable_vertex_attrib_array(color_attribute as u32);

    // Draw
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    let vert_count = (vertices.len() / 2) as i32;
    context.bind_vertex_array(Some(&vbo));
    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);

    context.bind_vertex_array(None);

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
