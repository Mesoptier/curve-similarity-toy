use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

mod color_maps;

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
        include_str!("shader.vert"),
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        include_str!("shader.frag"),
    )?;

    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    /// The function to plot
    fn f(x: f32, y: f32, t: f32) -> f32 {
        ((x * y * 10.0 + t).sin() + 1.0) / 2.0
    }

    const BYTES_PER_FLOAT: i32 = 4;

    const FLOATS_PER_POSITION: i32 = 2;
    const FLOATS_PER_VALUE: i32 = 1;
    const FLOATS_PER_VERTEX: i32 = FLOATS_PER_POSITION + FLOATS_PER_VALUE;

    fn build_vertex_data(x_len: i32, y_len: i32, t: f32) -> Vec<f32> {
        let vertex_data_len = ((x_len * y_len) * FLOATS_PER_VERTEX) as usize;
        let mut vertex_data: Vec<f32> = Vec::with_capacity(vertex_data_len);

        for y_idx in 0..y_len {
            for x_idx in 0..x_len {
                let x: f32 = (x_idx as f32 / (x_len - 1) as f32) * 2.0 - 1.0;
                let y: f32 = (y_idx as f32 / (y_len - 1) as f32) * 2.0 - 1.0;

                let v = f(x, y, t);

                // FLOATS_PER_POSITION
                vertex_data.push(x);
                vertex_data.push(y);

                // FLOATS_PER_VALUE
                vertex_data.push(v);
            }
        }

        assert_eq!(vertex_data.len(), vertex_data_len);

        vertex_data
    }

    fn build_index_data(x_len: i32, y_len: i32) -> Vec<u32> {
        let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;
        let mut index_data: Vec<u32> = Vec::with_capacity(index_data_len);

        for y in 0..(y_len - 1) {
            if y > 0 {
                // Degenerate begin: repeat first vertex
                index_data.push((y * x_len) as u32);
            }

            for x in 0..x_len {
                index_data.push(((y * x_len) + x) as u32);
                index_data.push((((y + 1) * x_len) + x) as u32);
            }

            if y < y_len - 2 {
                // Degenerate end: repeat last vertex
                index_data.push((((y + 1) * x_len) + (x_len - 1)) as u32);
            }
        }

        assert_eq!(index_data.len(), index_data_len);

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

    // Bind attributes
    let position_attribute =
        context.get_attrib_location(&program, "a_position") as u32;
    let value_attribute =
        context.get_attrib_location(&program, "a_value") as u32;

    let color_map_uniform = context
        .get_uniform_location(&program, "u_color_map")
        .unwrap();

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
        value_attribute,
        FLOATS_PER_VALUE,
        WebGl2RenderingContext::FLOAT,
        false,
        FLOATS_PER_VERTEX * BYTES_PER_FLOAT,
        FLOATS_PER_POSITION * BYTES_PER_FLOAT,
    );
    context.enable_vertex_attrib_array(value_attribute);

    // - Mesh of main triangles (note: constant value)
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

    // TODO: Use alternative vertex shader with a color uniform instead of a value attribute
    context.vertex_attrib1f(value_attribute, 0.0);

    // UI
    let show_mesh = Rc::new(Cell::new(false));
    let is_playing = Rc::new(Cell::new(true));

    let color_map_name = Rc::new(RefCell::new("magma".to_string()));
    let color_map_name_changed = Rc::new(Cell::new(true));

    let resolution = Rc::new(Cell::new(7i32));
    let resolution_changed = Rc::new(Cell::new(true));

    // Event listeners
    let document = web_sys::window().unwrap().document().unwrap();

    {
        let show_mesh = show_mesh.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            show_mesh.set(!show_mesh.get());
        });
        let button = document
            .get_element_by_id("toggle-show-mesh")
            .expect("Element with ID #toggle-show-mesh");
        button.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let is_playing = is_playing.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            is_playing.set(!is_playing.get());
        });
        let button = document
            .get_element_by_id("toggle-playing")
            .expect("Element with ID #toggle-playing");
        button.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let color_map_name = color_map_name.clone();
        let color_map_name_changed = color_map_name_changed.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                *color_map_name.borrow_mut() = event
                    .current_target()
                    .unwrap()
                    .dyn_into::<web_sys::HtmlSelectElement>()
                    .unwrap()
                    .value();
                color_map_name_changed.set(true);
            });
        let select = document
            .get_element_by_id("select-color-map")
            .expect("Element with ID #select-color-map");
        select.add_event_listener_with_callback(
            "change",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    {
        let resolution = resolution.clone();
        let resolution_changed = resolution_changed.clone();
        let closure =
            Closure::<dyn FnMut(_)>::new(move |event: web_sys::InputEvent| {
                resolution.set(
                    event
                        .current_target()
                        .unwrap()
                        .dyn_into::<web_sys::HtmlInputElement>()
                        .unwrap()
                        .value()
                        .parse()
                        .unwrap(),
                );
                resolution_changed.set(true);
            });
        let input = document
            .get_element_by_id("input-resolution")
            .expect("Element with ID #input-resolution");
        input.add_event_listener_with_callback(
            "input",
            closure.as_ref().unchecked_ref(),
        )?;
        closure.forget();
    }

    // Begin draw loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let color_map_name = color_map_name.clone();
    let color_map_name_changed = color_map_name_changed.clone();

    let mut frame = 0;
    *g.borrow_mut() = Some(Closure::new(move || {
        if is_playing.get() {
            frame += 1;
        }

        let resolution = 2i32.pow(resolution.get() as u32);
        let x_len = resolution;
        let y_len = resolution;

        let index_data_len = 2 * (x_len * (y_len - 1) + (y_len - 2)) as usize;

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

        // Update index data
        if resolution_changed.get() {
            resolution_changed.set(false);

            let index_data = build_index_data(x_len, y_len);

            context.bind_buffer(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                Some(&index_buffer),
            );

            unsafe {
                let index_data_array_buf_view =
                    js_sys::Uint32Array::view(&index_data);
                context.buffer_data_with_array_buffer_view(
                    WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                    &index_data_array_buf_view,
                    WebGl2RenderingContext::DYNAMIC_DRAW,
                )
            }
        }

        // TODO: Update index data if x_len/y_len changed

        // Update color map if the selected one changed
        if color_map_name_changed.get() {
            color_map_name_changed.set(false);

            let color_map = match color_map_name.borrow().as_str() {
                "magma" => color_maps::COLOR_MAP_MAGMA,
                "inferno" => color_maps::COLOR_MAP_INFERNO,
                "plasma" => color_maps::COLOR_MAP_PLASMA,
                "viridis" => color_maps::COLOR_MAP_VIRIDIS,
                _ => unreachable!("Invalid color map name"),
            };

            context.uniform3fv_with_f32_array(
                Some(&color_map_uniform),
                &color_map,
            );
        }

        // Draw
        context.clear_color(0.0, 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        context.bind_vertex_array(Some(&vao_triangles));
        context.draw_elements_with_i32(
            WebGl2RenderingContext::TRIANGLE_STRIP,
            index_data_len as i32,
            WebGl2RenderingContext::UNSIGNED_INT,
            0,
        );

        if show_mesh.get() {
            context.bind_vertex_array(Some(&vao_lines));
            context.draw_elements_with_i32(
                WebGl2RenderingContext::LINE_STRIP,
                index_data_len as i32,
                WebGl2RenderingContext::UNSIGNED_INT,
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
